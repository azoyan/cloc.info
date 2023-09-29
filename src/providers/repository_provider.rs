use super::git::{self, Git};
use crate::{
    cloner::Cloner,
    repository::{
        info::{to_unique_name, to_url, Branches, Status, Task},
        storage_cache::DiskCache,
        utils::{self, count_line_of_code},
    },
    Id,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dashmap::DashMap;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio_postgres::{IsolationLevel::Serializable, NoTls, Row};
use tracing::{error, info};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Can't deserialize bytes: {bytes} from request {url} {source}"))]
    DeserializeError {
        bytes: String,
        url: String,
        source: serde_json::Error,
    },

    #[snafu(display("Repository not found by API request {url}"))]
    NotFound { url: String },

    #[snafu(display("Error at API request {url} message: {message}"))]
    RemoteError { url: String, message: String },

    #[snafu(display("Can't extract default branch for repository {repo}"))]
    ExtractDefaultBranchError { repo: String },

    #[snafu(display("Template page not found"))]
    TemplatePage,

    #[snafu(display("Can't create request '{url}': {source}"))]
    CreateRequest {
        url: String,
        source: hyper::http::Error,
    },

    #[snafu(display("Can't send request '{url}': {source}"))]
    SendRequest { url: String, source: hyper::Error },

    #[snafu(display("Can't get response body '{url}': {source}"))]
    GetResponseBody { url: String, source: hyper::Error },

    #[snafu(display("Branch '{wrong_branch}' is note exist"))]
    WrongBranch { wrong_branch: String },

    #[snafu(display("Error {source} at query {query}"))]
    Query {
        query: String,
        source: tokio_postgres::Error,
    },

    #[snafu(display("Error at utils: {source}"))]
    UtilsError { source: utils::Error },

    #[snafu(display("Can't create temporary directory: {source}"))]
    CreateTempDirError { source: std::io::Error },

    #[snafu(display("Repository downloading already in progress"))]
    InProgress { url: String },

    #[snafu(display("Error at 'git ls-remote': {source}"))]
    GitProviderError { source: git::Error },
}

#[derive(Clone)]
pub struct RepositoryProvider {
    disk_cache: Arc<RwLock<DiskCache>>,
    pub connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub git_provider: Git,
    pub cloner: Cloner,
    notify: Arc<Notify>,
    tasks: Arc<Mutex<Vec<Task>>>,
    statuses: Arc<DashMap<String, Status>>,
    cancel: Arc<tokio_util::sync::CancellationToken>,
}

impl RepositoryProvider {
    pub fn new(
        cache_size: u64,
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
        git_provider: Git,
        cancel: Arc<tokio_util::sync::CancellationToken>,
    ) -> Self {
        let statuses = Arc::new(DashMap::with_capacity_and_shard_amount(512, 32));
        let tasks = Arc::new(Mutex::new(Vec::with_capacity(1024)));
        let cloner = Cloner::new(statuses.clone());
        Self {
            disk_cache: Arc::new(RwLock::new(DiskCache::new(cache_size))),
            connection_pool,
            git_provider,
            cloner,
            notify: Default::default(),
            tasks,
            statuses,
            cancel,
        }
    }

    pub async fn run(&self) {
        let mut tasks = Vec::with_capacity(1024);
        while !self.cancel.is_cancelled() {
            match self.tasks.try_lock() {
                Ok(mut t) => tasks.append(&mut t),
                Err(e) => error!("run() Lock error: {e}"),
            };

            while let Some(task) = tasks.pop() {
                self.process_task(task);
            }

            tokio::time::sleep(std::time::Duration::from_micros(50)).await;
        }
    }

    pub fn process_task(&self, task: Task) {
        let mut s = self.clone();
        let future = async move {
            let unique_name = task.to_unique_name();
            let user_agent = &task.user_agent;

            match s.process_task_impl(&unique_name, &task).await {
                Ok((branch_id, out)) => {
                    s.statuses.insert(unique_name, Status::Done(out));
                    s.update_statistic(branch_id, user_agent).await;
                }
                Err(e) => {
                    tracing::error!("Error at processing {unique_name}: {}", e);
                    s.statuses.insert(unique_name, Status::Error(e.to_string()));
                }
            }
        };
        tokio::spawn(future);
    }

    // Новый алгортим
    // 1. Смотрим есть ли в БД
    // 2. Если есть:
    // 2.1 проверяем актуальный ли коммит, если да - возвращаем данные из БД
    // 2.2. если коммит не актуален делаем git pull и пересчитываем, обновляем данные в БД и обнолвяем хранилище на жёстком диске
    // 3. Если отсутствует:
    // 3.1 клонируем репозиторий, вставляем в БД, вставляем в хранилище на жёстком диске.

    // Работа с дисковым хранилищем.
    // 1.Смотрим, есть ли в дисковом хранилище
    // 2. Если есть:
    // 2.1 смотрим есть ли нужная ветка
    // 2.1.1 если есть pull'им текущую ветку
    // 2.1.2 если нет клонируем ветку
    pub async fn process_task_impl(
        &mut self,
        unique_name: &str,
        task: &Task,
    ) -> Result<(Id, Vec<u8>), Error> {
        info!("process_task(unique_name: {unique_name}, task: {task:?})");
        let Task {
            host,
            owner,
            repository_name,
            branch,
            default_branch,
            ..
        } = task;

        let is_default_branch = branch == default_branch;

        // 1. Смотрим есть ли в БД
        let mut connection: bb8::PooledConnection<'_, PostgresConnectionManager<NoTls>> =
            match self.connection_pool.get().await {
                Ok(connection) => connection,
                Err(error) => {
                    match error {
                        bb8::RunError::User(user) => error!("{user}"),
                        bb8::RunError::TimedOut => error!("timeout error"),
                    }
                    panic!("Error at connection")
                }
            };

        let query = "select * from branches where name=$4 and repository_id=(select id from repositories where hostname=$1 and owner=$2 and repository_name=$3);";

        let row = connection
            .query_opt(query, &[host, owner, repository_name, branch])
            .await
            .with_context(|_e| QuerySnafu {
                query: query.to_owned(),
            })?;

        let (branch_id, scc_output) = match row {
            Some(row) => {
                info!("Repository {unique_name} exist in database");
                if self.is_commit_actual(&row, task).await? {
                    let scc_output: Vec<u8> = row.get("scc_output");
                    let branch_id: Id = row.get("id");
                    return Ok((branch_id, scc_output));
                } else {
                    let status = if self.disk_cache.read().await.contains(unique_name) {
                        info!("{repository_name} cached in disk: {}", unique_name);
                        self.cloner.pull_repository(task, unique_name).await
                    } else {
                        info!("Repository {repository_name} is not cached in disk");
                        self.cloner.clone_repository(task, unique_name).await
                    };

                    info!("Repository {unique_name} status: {status:?}");

                    let repository_size =
                        i64::try_from(utils::dir_size(unique_name).unwrap()).unwrap();
                    let scc_output = count_line_of_code(unique_name, "")
                        .await
                        .context(UtilsSnafu)?;
                    let last_commit_local =
                        utils::last_commit_local(unique_name).context(UtilsSnafu)?;

                    let branch_id = self
                        .insert_to_database(
                            &mut connection,
                            row,
                            task,
                            &scc_output,
                            &last_commit_local,
                            repository_size,
                        )
                        .await?;

                    (branch_id, scc_output)
                }
            }
            None => {
                info!("Repository {unique_name} doesn't exist in database and disk");

                let state = self.cloner.clone_repository(task, unique_name).await;
                info!("Clone state {state:?} for {unique_name}");

                let repository_size = utils::dir_size(unique_name).context(UtilsSnafu)?;
                let scc_output = count_line_of_code(unique_name, "")
                    .await
                    .context(UtilsSnafu)?;
                let last_commit_local =
                    utils::last_commit_local(unique_name).context(UtilsSnafu)?;

                let branch_id = self
                    .update_db(
                        &mut connection,
                        task,
                        &scc_output,
                        &last_commit_local,
                        unique_name,
                        repository_size as i64,
                    )
                    .await?;

                (branch_id, scc_output)
            }
        };
        if is_default_branch {
            self.disk_cache.write().await.insert(unique_name);
        } else {
            self.disk_cache
                .write()
                .await
                .remove(unique_name)
                .context(CreateTempDirSnafu)?;
        }

        Ok((branch_id, scc_output))
    }

    pub async fn add_task(
        &self,
        host: String,
        owner: String,
        repository_name: String,
        branch: Option<String>,
        user_agent: String,
    ) -> Result<String, Error> {
        info!("add_task {} {} {:?}", owner, repository_name, branch);

        let url = to_url(&host, &owner, &repository_name);

        let default_branch = self
            .git_provider
            .default_branch(&url)
            .await
            .with_context(|_e| GitProviderSnafu)?;

        let branch = branch.unwrap_or(default_branch.clone());

        let unique_name = to_unique_name(&host, &owner, &repository_name, &branch);

        let task = Task {
            host,
            owner,
            repository_name,
            branch,
            default_branch,
            user_agent,
        };

        match self.tasks.try_lock() {
            Ok(mut tasks) => {
                if !tasks
                    .iter()
                    .any(|task| unique_name == task.to_unique_name())
                {
                    tasks.push(task);
                    self.notify.notify_waiters();
                }
            }
            Err(e) => error!("Can't get lock to add task {}", e),
        }

        self.statuses.insert(unique_name.clone(), Status::Ready);

        Ok(unique_name)
    }

    pub fn current_status(&self, unique_name: &str) -> Status {
        self.statuses.get(unique_name).unwrap().value().clone()
    }

    pub async fn default_branch_remote(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
    ) -> Result<String, Error> {
        let url = to_url(host, owner, repository_name);
        let default_branch = self
            .git_provider
            .default_branch(&url)
            .await
            .with_context(|_e| GitProviderSnafu)?;
        Ok(default_branch)
    }

    pub async fn last_commit_remote(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
        branch: &str,
    ) -> Result<String, Error> {
        let url = to_url(host, owner, repository_name);
        let branch = branch.trim_start_matches('/');
        let last_commit = self
            .git_provider
            .last_commit(&url, branch)
            .await
            .with_context(|_e| GitProviderSnafu)?;
        Ok(last_commit)
    }

    pub async fn remote_branches(&self, url: &str) -> Result<Branches, Error> {
        let branches = self
            .git_provider
            .all_branches(url)
            .await
            .with_context(|_e| GitProviderSnafu)?;
        Ok(branches)
    }

    async fn update_statistic(&self, branch_id: Id, user_agent: &str) {
        let connection = self.connection_pool.get().await.unwrap();
        let query = "INSERT INTO statistic VALUES(DEFAULT, $1, $2, NOW());";
        let r = connection
            .execute(query, &[&user_agent, &branch_id])
            .await
            .with_context(|_e| QuerySnafu {
                query: query.to_string(),
            });

        match r {
            Ok(row_modified) => {
                info!("Insert to statistic. Row modified {row_modified}")
            }
            Err(error) => error!("Insert statistic error: {}", error.to_string()),
        }
    }

    async fn insert_to_database(
        &self,
        connection: &mut bb8::PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        row: Row,
        Task { branch, .. }: &Task,
        scc_output: &[u8],
        last_commit_local: &str,
        repository_size: i64,
    ) -> Result<Id, Error> {
        let repository_id: Id = row.get("repository_id");
        tracing::debug!(
                "INSERT INTO branches VALUES(DEFAULT, {}, '{}', '{}', 'scc', {}) ON CONFLICT (repository_id, name) DO UPDATE SET repository_id = EXCLUDED.repository_id, name = EXCLUDED.name, last_commit_sha = EXCLUDED.last_commit_sha RETURNING id;",
                repository_id,
                branch,
                last_commit_local,
                repository_size
            );

        let upsert_branch = "INSERT INTO branches VALUES(DEFAULT, $1, $2, $3, $4, $5) ON CONFLICT (repository_id, name) DO UPDATE SET repository_id = EXCLUDED.repository_id, name = EXCLUDED.name, last_commit_sha = EXCLUDED.last_commit_sha RETURNING id";
        let transaction = connection
            .build_transaction()
            .isolation_level(Serializable)
            .start()
            .await
            .with_context(|_e| QuerySnafu {
                query: upsert_branch.to_string(),
            })?;
        transaction
            .query_one(
                upsert_branch,
                &[
                    &repository_id,
                    branch,
                    &last_commit_local,
                    &scc_output,
                    &repository_size,
                ],
            )
            .await
            .with_context(|_e| QuerySnafu {
                query: upsert_branch.to_string(),
            })?;

        transaction.commit().await.with_context(|_e| QuerySnafu {
            query: upsert_branch.to_string(),
        })?;

        let branch_id = row.get("id");
        Ok(branch_id)
    }

    async fn update_db(
        &self,
        connection: &mut bb8::PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        Task {
            host,
            owner,
            repository_name,
            branch,
            default_branch,
            ..
        }: &Task,
        scc_output: &[u8],
        last_commit_local: &str,
        path: &str,
        repository_size: i64,
    ) -> Result<Id, Error> {
        let upsert_repositories = "insert into repositories values (DEFAULT, $1, $2, $3, $4, $5) ON CONFLICT (hostname, owner, repository_name) DO UPDATE SET hostname=EXCLUDED.hostname, owner=EXCLUDED.owner, repository_name=EXCLUDED.repository_name , destination=EXCLUDED.destination RETURNING ID;";
        let transaction = connection
            .build_transaction()
            .isolation_level(Serializable)
            .start()
            .await
            .with_context(|_e| QuerySnafu {
                query: upsert_repositories.to_string(),
            })?;
        let row = transaction
            .query_one(
                upsert_repositories,
                &[host, owner, repository_name, default_branch, &path],
            )
            .await
            .with_context(|_e| QuerySnafu {
                query: upsert_repositories.to_string(),
            })?;

        transaction.commit().await.with_context(|_e| QuerySnafu {
            query: upsert_repositories.to_string(),
        })?;

        let repository_id: Id = row.get("id");

        tracing::debug!(
            "INSERT INTO branches VALUES(DEFAULT, {}, '{}', '{}', 'scc', {}) ON CONFLICT (repository_id, name) DO UPDATE SET repository_id = EXCLUDED.repository_id, name = EXCLUDED.name, last_commit_sha = EXCLUDED.last_commit_sha RETURNING id;",
            repository_id,
            branch,
            &last_commit_local,
            repository_size
        );

        let insert_branch = "INSERT INTO branches VALUES(DEFAULT, $1, $2, $3, $4, $5) RETURNING id";
        let transaction = connection
            .build_transaction()
            .isolation_level(Serializable)
            .start()
            .await
            .with_context(|_e| QuerySnafu {
                query: insert_branch.to_string(),
            })?;

        let row = transaction
            .query_one(
                insert_branch,
                &[
                    &repository_id,
                    branch,
                    &last_commit_local,
                    &scc_output,
                    &repository_size,
                ],
            )
            .await
            .with_context(|_e| QuerySnafu {
                query: insert_branch.to_string(),
            })?;

        transaction.commit().await.with_context(|_e| QuerySnafu {
            query: insert_branch.to_string(),
        })?;
        info!("Updating info for {path} to database done");
        let branch_id = row.get("id");

        Ok(branch_id)
    }

    async fn is_commit_actual(
        &self,
        row: &Row,
        Task {
            host,
            owner,
            repository_name,
            branch,
            ..
        }: &Task,
    ) -> Result<bool, Error> {
        let db_last_commit: String = row.get("last_commit_sha");
        let db_branch_name: String = row.get("name");

        let last_commit_remote = self
            .last_commit_remote(host, owner, repository_name, branch)
            .await?;

        let is_actual = last_commit_remote.eq(&db_last_commit) && db_branch_name.eq(branch);
        if is_actual {
            info!("Current branch and commit are actual. Returning cloc from db");
        } else {
            info!("Current branch '{db_branch_name}' and commit '{db_last_commit}' are not actual '{last_commit_remote}'");
        }

        Ok(is_actual)
    }
}
