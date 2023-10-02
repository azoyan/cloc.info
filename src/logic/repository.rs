use super::{
    cloner::Cloner,
    git::Git,
    info::{to_unique_name, to_url, Branches, Status, Task},
    Error, Id, QuerySnafu,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dashmap::DashMap;
use rand::{thread_rng, Rng};
use scopeguard::defer;
use snafu::ResultExt;
use std::{
    path::Path,
    process::{Command, Stdio},
    str::from_utf8,
    sync::Arc,
};
use tokio::sync::{Mutex, Notify};
use tokio_postgres::{IsolationLevel::Serializable, NoTls, Row};
use tracing::{error, info};

#[derive(Clone)]
pub struct RepositoryProvider {
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
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
        git_provider: Git,
        cancel: Arc<tokio_util::sync::CancellationToken>,
    ) -> Self {
        let statuses = Arc::new(DashMap::with_capacity_and_shard_amount(512, 32));
        let tasks = Arc::new(Mutex::new(Vec::with_capacity(1024)));
        let cloner = Cloner::new(statuses.clone());

        Self {
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
            ..
        } = task;
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
            .context(QuerySnafu { query })?;

        if let Some(row) = &row {
            info!("Repository {unique_name} exist in database");
            if self.is_commit_actual(row, task).await? {
                let scc_output: Vec<u8> = row.get("scc_output");
                let branch_id: Id = row.get("id");
                return Ok((branch_id, scc_output));
            }
        };
        info!("Repository {unique_name} doesn't exist in database");
        let tmp_path = generate_path();
        self.cloner
            .clone_repository(task, unique_name, &tmp_path)
            .await?;
        let scc_output = count_line_of_code(&tmp_path, "").await?;
        self.statuses
            .insert(unique_name.to_string(), Status::Done(scc_output.clone()));

        defer!(remove_dir(&tmp_path).expect("Can't remove dir"));
        let last_commit_local = last_commit_local(&tmp_path)?;
        let repository_size = dir_size(&tmp_path)? as i64;

        let branch_id = if let Some(row) = row {
            self.update_database(
                &mut connection,
                row,
                task,
                &scc_output,
                &last_commit_local,
                repository_size,
            )
            .await?
        } else {
            self.insert_to_database(
                &mut connection,
                task,
                &scc_output,
                &last_commit_local,
                unique_name,
                repository_size,
            )
            .await?
        };

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

        let default_branch = self.git_provider.default_branch(&url).await?;

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
        let default_branch = self.git_provider.default_branch(&url).await?;
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
        let last_commit = self.git_provider.last_commit(&url, branch).await?;
        Ok(last_commit)
    }

    pub async fn remote_branches(&self, url: &str) -> Result<Branches, Error> {
        let branches = self.git_provider.all_branches(url).await?;
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

    async fn update_database(
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

    async fn insert_to_database(
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
        let upsert_repositories = "insert into repositories values (DEFAULT, $1, $2, $3, $4) ON CONFLICT (hostname, owner, repository_name) DO UPDATE SET hostname=EXCLUDED.hostname, owner=EXCLUDED.owner, repository_name=EXCLUDED.repository_name RETURNING ID;";
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
                &[host, owner, repository_name, default_branch],
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

use String as CommitHash;

pub(crate) fn last_commit_local(path: &str) -> Result<CommitHash, Error> {
    let mut last_commit_command = std::process::Command::new("git");
    last_commit_command.args(["-C", path, "rev-parse", "HEAD"]);

    match last_commit_command.output() {
        Ok(output) if output.status.success() => match from_utf8(&output.stdout) {
            Ok(hash) => Ok(hash.trim().to_string()),
            Err(e) => Err(Error::LastCommitError {
                repository: path.to_string(),
                error: e.to_string(),
            }),
        },
        Ok(output) => {
            let status_code = match output.status.code() {
                Some(code) => code.to_string(),
                None => String::from("None"),
            };

            Err(Error::LastCommitError {
                repository: path.to_string(),
                error: format!("git status code: {status_code}"),
            })
        }
        Err(e) => Err(Error::LastCommitError {
            repository: path.to_string(),
            error: e.to_string(),
        }),
    }
}

pub async fn count_line_of_code(path: &str, _format: &str) -> Result<Vec<u8>, Error> {
    let mut scc_command = tokio::process::Command::new("scc");
    tracing::debug!("Counting line of code in path: {path}");
    scc_command.args(["--ci", path]);
    let out = match scc_command.output().await {
        Ok(output) if !output.status.success() => {
            let error = String::from_utf8(output.stderr)
                .unwrap_or_else(|e| format!("Error at convert git output to utf8: {e}"));
            tracing::error!("scc error: {}", error);
            return Err(Error::SccError { error });
        }
        Ok(output) => output.stdout,
        Err(e) => {
            tracing::error!("scc error: {e}");
            return Err(Error::SccError {
                error: e.to_string(),
            });
        }
    };
    // tracing::info!("{:?}", out);
    Ok(out)
}

fn remove_dir(path: &str) -> Result<(), std::io::Error> {
    Command::new("rm").args(["-rf", path, "&"]).status()?;
    tracing::debug!("Remove {path}");
    Ok(())
}

pub fn dir_size<P>(path: P) -> Result<u64, Error>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_str().unwrap();
    let mut command = Command::new("du");
    command.args(["-sb", path]);

    command.stdout(Stdio::piped());

    match command.output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(out) => match out.split_whitespace().next() {
                Some(str) => Ok(str.parse::<u64>().map_err(|e| Error::Size {
                    error: e.to_string(),
                }))?,
                None => Err(Error::Size {
                    error: format!("Wrong `du` stdout: {out}"),
                }),
            },
            Err(e) => Err(Error::Size {
                error: e.to_string(),
            }),
        },
        Err(e) => Err(Error::Size {
            error: e.to_string(),
        }),
    }
}

fn generate_path() -> String {
    use rand::distributions::Alphanumeric;
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
}
