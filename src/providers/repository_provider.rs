use super::git_provider::{self, GitProvider};
use crate::{
    cloner::Cloner,
    repository::{
        info::{to_unique_name, to_url, Branches},
        storage_cache::StorageCache,
        utils::{self, count_line_of_code},
    },
    DbId,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use scopeguard::defer;
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI64, Ordering::SeqCst},
        Arc,
    },
};
use tempfile::TempDir;
use tokio::sync::{Notify, RwLock};
use tokio_postgres::{IsolationLevel::Serializable, NoTls};

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

    #[snafu(display("Error at getting last commit: {source}"))]
    LastCommitError { source: utils::Error },

    #[snafu(display("Error at scc: {source}"))]
    SccError { source: utils::Error },

    #[snafu(display("Error at cloning repository or scc: {source}"))]
    DownloaderError { source: utils::Error },

    #[snafu(display("Can't create temporary directory: {source}"))]
    CreateTempDirError { source: std::io::Error },

    #[snafu(display("Repository downloading already in progress"))]
    InProgress { url: String },

    #[snafu(display("Error at 'git ls-remote': {source}"))]
    GitProviderError { source: git_provider::Error },
}

#[derive(Debug, Default)]
struct VisitCounter {
    counter: AtomicI64,
    notify: Notify,
}

#[derive(Debug, Clone)]
pub struct Processed {
    branch_id: DbId,
    scc_output: Vec<u8>,
    directory: Option<Arc<TempDir>>, // need to drop after all processing done // TODO refactor
}
#[derive(Clone)]
pub struct RepositoryProvider {
    storage_cache: Arc<RwLock<StorageCache>>,
    pub connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub git_provider: GitProvider,
    pub cloner: Cloner,
    processed_repostitoires: Arc<RwLock<HashMap<String, Arc<VisitCounter>>>>,
}

impl AsRef<RepositoryProvider> for RepositoryProvider {
    fn as_ref(&self) -> &RepositoryProvider {
        self
    }
}

impl RepositoryProvider {
    pub fn new(
        cache_size: u64,
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
        git_provider: GitProvider,
    ) -> Self {
        Self {
            storage_cache: Arc::new(RwLock::new(StorageCache::new(cache_size))),
            connection_pool,
            git_provider,
            cloner: Cloner::new(),
            processed_repostitoires: Default::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn processing(
        &self,
        unique_name: &str,
        host: &str,
        owner: &str,
        repository_name: &str,
        branch: &str,
        default_branch: &str,
        user_agent: &str,
    ) -> Result<Processed, Error> {
        // Узнаем какой коммит был последний
        let last_commit_remote = self
            .last_commit_remote(host, owner, repository_name, branch)
            .await?;
        let last_commit_local;

        let is_default_branch = branch == default_branch;

        // Новый алгортим
        // 1. Смотрим есть ли в БД
        // 2. Если есть:
        // 2.1 проверяем актуальный ли коммит, если да - возвращаем данные из БД
        // 2.2. если коммит не актуален делаем git pull и пересчитываем, обновляем данные в БД и обнолвяем хранилище на жёстком диске
        // 3. Если отсутствует:
        // 3.1 клонируем репозиторий, вставляем в БД, вставляем в хранилище на жёстком диске.

        // 1. Смотрим есть ли в БД
        let mut connection = match self.connection_pool.get().await {
            Ok(connection) => connection,
            Err(error) => {
                match error {
                    bb8::RunError::User(user) => tracing::error!("{}", user.to_string()),
                    bb8::RunError::TimedOut => tracing::error!("timeout error"),
                }
                panic!("Error at connection")
            }
        };

        let query = "select * from branches where name=$4 and repository_id=(select id from repositories where hostname=$1 and owner=$2 and repository_name=$3);";

        let row = connection
            .query_opt(query, &[&host, &owner, &repository_name, &branch])
            .await
            .with_context(|_e| QuerySnafu {
                query: query.to_owned(),
            })?;

        let processed = match row {
            // Если в БД присутствует
            Some(row) => {
                // проверяем актуальность коммита
                let db_last_commit: String = row.get("last_commit_sha");
                let db_branch_name: String = row.get("name");
                let branch_id: DbId = row.get("id");

                if last_commit_remote == db_last_commit && db_branch_name == branch {
                    let scc_output: Vec<u8> = row.get("scc_output");
                    tracing::info!("Current branch and commit are actual. Returning cloc from db");
                    Processed {
                        branch_id,
                        scc_output,
                        directory: None,
                    }
                } else {
                    tracing::info!("Current branch '{db_branch_name}' and commit '{db_last_commit}' are not actual '{last_commit_remote}'");

                    let (result, temp_dir) = match self
                        .storage_cache
                        .write()
                        .await
                        .take(unique_name)
                    {
                        Some(temp_dir) => {
                            let repository_path = temp_dir.path().to_str().unwrap();
                            tracing::info!("Repository {repository_name} cached in disk storage: {repository_path}");
                            // если есть, обновляем репозиторий
                            let result = self
                                .cloner
                                .pull_repository(
                                    host,
                                    owner,
                                    repository_name,
                                    repository_path,
                                    branch,
                                )
                                .await;
                            last_commit_local =
                                utils::last_commit_local(unique_name, repository_path)
                                    .with_context(|_e| LastCommitSnafu)?;
                            (result, temp_dir.clone())
                        }
                        None => {
                            tracing::info!(
                                "Repository {repository_name} is not cached in disk storage"
                            );
                            let temp_dir =
                                Arc::new(TempDir::new_in("cloc_repo").context(CreateTempDirSnafu)?);
                            let repository_path = temp_dir.path();
                            let repository_path_str = repository_path.to_str().unwrap();
                            let result = self
                                .cloner
                                .clone_repository(
                                    host,
                                    owner,
                                    repository_name,
                                    branch,
                                    repository_path_str,
                                )
                                .await;
                            last_commit_local =
                                utils::last_commit_local(unique_name, repository_path_str)
                                    .with_context(|_e| LastCommitSnafu)?;
                            (result, temp_dir)
                        }
                    };

                    tracing::info!(
                        "Repository {repository_name} cloned state: {:?} Path: {:?}",
                        result,
                        temp_dir.path()
                    );
                    let repository_path = temp_dir.path().to_str().unwrap();
                    let repository_size =
                        i64::try_from(fs_extra::dir::get_size(temp_dir.path()).unwrap()).unwrap();
                    let cloc = count_line_of_code(repository_path, "")
                        .await
                        .context(SccSnafu)?;
                    let repository_id: DbId = row.get("repository_id");

                    tracing::debug!(
                        "INSERT INTO branches VALUES(DEFAULT, {}, '{}', '{}', 'scc', {}) ON CONFLICT (repository_id, name) DO UPDATE SET repository_id = EXCLUDED.repository_id, name = EXCLUDED.name, last_commit_sha = EXCLUDED.last_commit_sha RETURNING id;",
                        repository_id,
                        &branch,
                        &last_commit_local,
                        repository_size
                    );

                    let branch_id = {
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
                                    &branch,
                                    &last_commit_local,
                                    &cloc,
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

                        row.get("id")
                    };

                    if is_default_branch {
                        self.storage_cache
                            .write()
                            .await
                            .insert(unique_name, temp_dir.clone());
                    }
                    Processed {
                        branch_id,
                        scc_output: cloc,
                        directory: Some(temp_dir),
                    }
                }
            }
            // Если в БД отсутствует
            None => {
                tracing::warn!(
                    "Repository {unique_name} doesn't exist in database and storage cache"
                );
                {
                    assert!(self.storage_cache.write().await.take(unique_name).is_none());
                }

                let temp_dir = Arc::new(TempDir::new_in("cloc_repo").context(CreateTempDirSnafu)?);
                let repository_path = temp_dir.path();
                let repository_path_str = repository_path.to_str().unwrap();
                // клонируем репозиторий

                let state = self
                    .cloner
                    .clone_repository(host, owner, repository_name, branch, repository_path_str)
                    .await;

                tracing::info!(
                    "Clone state {state:?} for {unique_name}, path: {:?}",
                    repository_path
                );

                // std::thread::sleep(std::time::Duration::from_secs(25));

                let repository_size = match fs_extra::dir::get_size(repository_path) {
                    Ok(size) => match i64::try_from(size) {
                        Ok(size) => size,
                        Err(e) => {
                            tracing::error!("{}. Repository will be set 0", e.to_string());
                            0
                        }
                    },
                    Err(e) => {
                        tracing::error!("{}. Repository will be set 0", e.to_string());
                        0
                    }
                };

                last_commit_local = utils::last_commit_local(unique_name, repository_path_str)
                    .with_context(|_e| LastCommitSnafu)?;
                let scc_output = count_line_of_code(repository_path_str, "")
                    .await
                    .context(SccSnafu)?;

                // вставляем результат в БД

                let repository_id: DbId = {
                    let upsert_repositories = "insert into repositories values (DEFAULT, $1, $2, $3, $4) ON CONFLICT (hostname, owner, repository_name) DO UPDATE SET hostname=EXCLUDED.hostname, owner=EXCLUDED.owner, repository_name=EXCLUDED.repository_name  RETURNING ID;";
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
                            &[&host, &owner, &repository_name, &default_branch],
                        )
                        .await
                        .with_context(|_e| QuerySnafu {
                            query: upsert_repositories.to_string(),
                        })?;

                    transaction.commit().await.with_context(|_e| QuerySnafu {
                        query: upsert_repositories.to_string(),
                    })?;

                    row.get("id")
                };

                tracing::debug!(
                    "INSERT INTO branches VALUES(DEFAULT, {}, '{}', '{}', 'scc', {}) ON CONFLICT (repository_id, name) DO UPDATE SET repository_id = EXCLUDED.repository_id, name = EXCLUDED.name, last_commit_sha = EXCLUDED.last_commit_sha RETURNING id;",
                    repository_id,
                    &branch,
                    &last_commit_local,
                    repository_size
                );

                let branch_id = {
                    let insert_branch =
                        "INSERT INTO branches VALUES(DEFAULT, $1, $2, $3, $4, $5) RETURNING id";
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
                                &branch,
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

                    row.get("id")
                };
                tracing::info!(
                    "Inserting {unique_name} info to database done. Returning scc_output"
                );
                // добовалем в хранилище на жёстком диске
                if is_default_branch {
                    self.storage_cache
                        .write()
                        .await
                        .insert(unique_name, temp_dir.clone());
                }
                Processed {
                    branch_id,
                    scc_output,
                    directory: Some(temp_dir),
                }
            }
        };

        if is_default_branch {
            self.update_statistic(processed.branch_id, user_agent).await;
        }

        Ok(processed)
    }

    pub async fn get_with_branch(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
        branch: Option<&str>,
        user_agent: &str,
    ) -> Result<(DbId, Vec<u8>), Error> {
        tracing::debug!(
            "get_with_branch({} {} {:?})",
            owner,
            repository_name,
            branch
        );
        let url = to_url(host, owner, repository_name);
        let default_branch = self
            .git_provider
            .default_branch(&url)
            .await
            .with_context(|_e| GitProviderSnafu)?;

        let branch = match branch {
            Some(branch) => branch,
            None => &default_branch,
        };
        let unique_name = to_unique_name(host, owner, repository_name, branch);
        // self.cloner.clear_state_buffer(&url).await;  // try remove later and check progress view
        defer! {
            match self.processed_repostitoires.try_read() {
                Ok(guard) => {
                    if let Some(visit) = guard.get(&unique_name) {
                        tracing::debug!("Trying Remove notificator for {unique_name}");
                        let counter = visit.counter.load(SeqCst);
                        assert!(counter >=0);
                        if counter == 0 {
                            drop(guard);
                            match self.processed_repostitoires.try_write() {
                                Ok(mut guard) => {
                                    guard.remove(&unique_name);
                                    tracing::debug!("Remove Done notificator for {unique_name}");

                                },
                                Err(e) =>  tracing::error!(
                                    "Remove notificator Error at write lock for {unique_name}: {e}"
                                ),
                            }
                        }
                        else {
                            let prev = visit.counter.fetch_sub(1, SeqCst);
                            tracing::debug!("Can't Remove notificator for {unique_name}, visitors: {}", prev);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Remove notificator Error at read lock for {unique_name}: {e}"
                    );
                }
            }

            tracing::info!("Out of the scope {unique_name} get_with_branch\n");
        }

        match self.processed_repostitoires.try_read() {
            Ok(guard) => match guard.get(&unique_name) {
                None => {
                    tracing::debug!("First processing repository {unique_name}");
                    drop(guard);
                    match self.processed_repostitoires.try_write() {
                        Ok(mut guard) => {
                            guard.insert(
                                unique_name.clone(),
                                Arc::new(VisitCounter {
                                    counter: AtomicI64::new(0),
                                    notify: Default::default(),
                                }),
                            );
                        }
                        Err(e) => tracing::error!(
                            "First notificator Error at write lock for {unique_name}: {}",
                            e
                        ),
                    }
                }
                Some(notify) => {
                    let visit = notify.clone();
                    drop(guard);
                    let prev = visit.counter.fetch_add(1, SeqCst);
                    tracing::debug!(
                        "Someone else ({}) request but In prgress {unique_name} ...",
                        prev + 1
                    );
                    visit.notify.notified().await;
                    // tracing::debug!("Continue {url} progress...");
                }
            },
            Err(e) => {
                tracing::error!("Someone else Error at read lock for {unique_name}: {}", e);
            }
        }

        // tracing::info!("Start Processing {url}");
        let Processed {
            branch_id,
            scc_output,
            directory: _directory,
        } = self
            .processing(
                &unique_name,
                host,
                owner,
                repository_name,
                branch,
                &default_branch,
                user_agent,
            )
            .await?;

        // tracing::info!("End Processing {url}. Done: {}", result.is_ok());

        match self.processed_repostitoires.try_read() {
            Ok(guard) => match guard.get(&unique_name) {
                Some(notify) => {
                    // tracing::debug!("Processing {url} done. Notify to other waiters");
                    self.cloner.set_done(&unique_name).await;
                    let n = notify.clone();
                    drop(guard);
                    n.notify.notify_waiters();
                }
                None => {
                    // tracing::debug!("Processing {url} done. No other waiters");
                }
            },
            Err(_e) => {
                // tracing::error!("Processing notificator Error at write lock for {url}: {e}");
            }
        }

        Ok((branch_id, scc_output))
    }

    pub async fn default_branch_remote(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
    ) -> Result<String, Error> {
        let url = format!("https://{host}/{owner}/{repository_name}");
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
        let url = format!("https://{host}/{owner}/{repository_name}");
        let branch = branch.trim_start_matches('/');
        let last_commit = self
            .git_provider
            .last_commit(&url, branch)
            .await
            .with_context(|_e| GitProviderSnafu)?;
        Ok(last_commit)
    }

    pub async fn remote_branches(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
    ) -> Result<Branches, Error> {
        let url = format!("https://{host}/{owner}/{repository_name}");
        let branches = self
            .git_provider
            .all_branches(&url)
            .await
            .with_context(|_e| GitProviderSnafu)?;
        Ok(branches)
    }

    async fn update_statistic(&self, branch_id: DbId, user_agent: &str) {
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
                tracing::info!("Insert to statistic. Row modified {row_modified}")
            }
            Err(error) => tracing::error!("Insert statistic error: {}", error.to_string()),
        }
    }
}
