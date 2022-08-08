use crate::{
    cloner::Cloner,
    repository::{
        info::{to_url, BranchInfo, BranchValue, Branches, RepositoryInfo},
        storage_cache::StorageCache,
        utils::{self, count_line_of_code},
    },
    DbId,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use hyper::{Body, Request};
use hyper_openssl::HttpsConnector;
use retainer::Cache;
use scopeguard::defer;
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI64, Ordering::SeqCst},
        Arc,
    },
    time::Duration,
};
use tempfile::TempDir;
use tokio::sync::{Notify, RwLock};
use tokio_postgres::{IsolationLevel::Serializable, NoTls};

const USERNAME_TOKEN: &str =
    "Basic YXpveWFuOmdocF9IOEVqSXRwMjBOQW9Gc3dYVGI4ektEaktUbkFETlg0TktaNUk=";

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

pub struct GithubProvider {
    storage_cache: Arc<RwLock<StorageCache>>,
    pub connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub cache: Arc<Cache<RepositoryInfo, Branches>>,
    pub cloner: Cloner,
    processed: Arc<RwLock<HashMap<String, Arc<VisitCounter>>>>,
}

impl GithubProvider {
    pub fn new(
        cache_size: u64,
        connection_pool: Pool<PostgresConnectionManager<NoTls>>,
        cache: Arc<Cache<RepositoryInfo, Branches>>,
    ) -> Self {
        Self {
            storage_cache: Arc::new(RwLock::new(StorageCache::new(cache_size))),
            connection_pool,
            cache,
            cloner: Cloner::new(),
            processed: Default::default(),
        }
    }

    pub async fn processing(
        &self,
        url: &str,
        owner: &str,
        repository_name: &str,
        branch: &str,
        default_branch: &str,
    ) -> Result<Processed, Error> {
        // Узнаем какой коммит был последний
        let last_commit_remote = self
            .last_commit_remote(owner, repository_name, branch)
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
            .query_opt(query, &[&"github.com", &owner, &repository_name, &branch])
            .await
            .with_context(|_e| QuerySnafu {
                query: query.to_owned(),
            })?;

        let processed = match row {
            // Есть в БД
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

                    let (result, temp_dir) = match self.storage_cache.read().await.get(url) {
                        Some(temp_dir) => {
                            let repository_path = temp_dir.path().to_str().unwrap();
                            tracing::info!("Repository {repository_name} cached in disk storage: {repository_path}");
                            // если есть, обновляем репозиторий
                            let result = self
                                .cloner
                                .pull_repository(url, repository_path, branch)
                                .await;
                            last_commit_local = utils::last_commit_local(url, repository_path)
                                .with_context(|_e| LastCommitSnafu)?;
                            (result, temp_dir.clone())
                        }
                        None => {
                            tracing::info!(
                                "Repository {repository_name} is not cached in disk storage"
                            );
                            let temp_dir =
                                Arc::new(TempDir::new_in("cloc_repo").context(CreateTempDirSnafu)?);
                            let repository_path = temp_dir.path().to_str().unwrap();
                            let result = self
                                .cloner
                                .clone_repository(url, branch, temp_dir.path().to_str().unwrap())
                                .await;
                            last_commit_local = utils::last_commit_local(url, repository_path)
                                .with_context(|_e| LastCommitSnafu)?;
                            (result, temp_dir)
                        }
                    };

                    tracing::info!("Repository {repository_name} cloned state: {:?}", result);
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
                            .insert(url, temp_dir.clone());
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
                tracing::warn!("Repository {url} doesn't exist in database and storage cache");
                assert!(self.storage_cache.read().await.get(url).is_none());

                let temp_dir = Arc::new(TempDir::new_in("cloc_repo").context(CreateTempDirSnafu)?);
                let repository_path = temp_dir.path().to_str().unwrap();
                // клонируем репозиторий
                
                let _state = self
                .cloner
                    .clone_repository(url, branch, repository_path)
                    .await;

                tracing::info!("Cloning {url} done");
                
                let repository_size =
                    i64::try_from(fs_extra::dir::get_size(temp_dir.path()).unwrap()).unwrap();
                
                last_commit_local = utils::last_commit_local(url, repository_path)
                    .with_context(|_e| LastCommitSnafu)?;
                let scc_output = count_line_of_code(repository_path, "")
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
                            &[&"github.com", &owner, &repository_name, &default_branch],
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
                tracing::info!("Inserting {url} info to database done. Returning scc_output");
                // добовалем в хранилище на жёстком диске
                if is_default_branch {
                    self.storage_cache
                        .write()
                        .await
                        .insert(url, temp_dir.clone());
                }
                Processed {
                    branch_id,
                    scc_output,
                    directory: Some(temp_dir),
                }
            }
        };

        Ok(processed)
    }

    pub async fn get_with_branch(
        &self,
        owner: &str,
        repository_name: &str,
        branch: Option<&str>,
    ) -> Result<(DbId, Vec<u8>), Error> {
        let url = format!("https://github.com/{owner}/{repository_name}");
        tracing::debug!(
            "\nGet with branch: {} {} {:?}",
            owner,
            repository_name,
            branch
        );

        let default_branch = self.default_branch_remote(owner, repository_name).await?;

        let branch = match branch {
            Some(branch) => branch,
            None => &default_branch,
        };
        // self.cloner.clear_state_buffer(&url).await;  // try remove later and check progress view
        defer! {
            match self.processed.try_read() {
                Ok(guard) => {
                    match guard.get(&url) {
                        Some(visit) => {
                            tracing::debug!("Trying Remove notificator for {url}");
                            let counter = visit.counter.load(SeqCst);
                            assert!(counter >=0);
                            if counter == 0 {
                               drop(guard);
                               match self.processed.try_write() {
                                    Ok(mut guard) => {
                                        guard.remove(&url);
                                        tracing::debug!("Remove Done notificator for {url}");

                                    },
                                    Err(e) =>  tracing::error!(
                                        "Remove notificator Error at write lock for {url}: {e}"
                                    ),
                                }
                            }
                            else {
                                let prev = visit.counter.fetch_sub(1, SeqCst);
                                tracing::debug!("Can't Remove notificator for {url}, visitors: {}", prev);
                            }
                        },
                        None => {},
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Remove notificator Error at read lock for {url}: {e}"
                    );
                }
            }

            tracing::info!("Out of the scope {url} get_with_branch\n");
        }

        match self.processed.try_read() {
            Ok(guard) => match guard.get(&url) {
                None => {
                    tracing::debug!("First processing repository {url}");
                    drop(guard);
                    match self.processed.try_write() {
                        Ok(mut guard) => {
                            guard.insert(
                                url.clone(),
                                Arc::new(VisitCounter {
                                    counter: AtomicI64::new(0),
                                    notify: Default::default(),
                                }),
                            );
                        }
                        Err(e) => tracing::error!(
                            "First notificator Error at write lock for {url}: {}",
                            e
                        ),
                    }
                }
                Some(notify) => {
                    let visit = notify.clone();
                    drop(guard);
                    let prev = visit.counter.fetch_add(1, SeqCst);
                    tracing::debug!(
                        "Someone else ({}) request but In prgress {url} ...",
                        prev + 1
                    );
                    visit.notify.notified().await;
                    // tracing::debug!("Continue {url} progress...");
                }
            },
            Err(e) => {
                tracing::error!("Someone else Error at read lock for {url}: {}", e);
            }
        }

        // tracing::info!("Start Processing {url}");
        let Processed {
            branch_id,
            scc_output,
            directory: _directory,
        } = self
            .processing(&url, owner, repository_name, branch, &default_branch)
            .await?;

        // tracing::info!("End Processing {url}. Done: {}", result.is_ok());

        match self.processed.try_read() {
            Ok(guard) => match guard.get(&url) {
                Some(notify) => {
                    // tracing::debug!("Processing {url} done. Notify to other waiters");
                    self.cloner.set_done(&url).await;
                    let n = notify.clone();
                    drop(guard);
                    n.notify.notify_waiters();
                }
                None => {
                    // tracing::debug!("Processing {url} done. No other waiters");
                }
            },
            Err(e) => {
                // tracing::error!("Processing notificator Error at write lock for {url}: {e}");
            }
        }

        Ok((branch_id, scc_output))
    }

    pub async fn default_branch_remote(
        &self,
        owner: &str,
        repository_name: &str,
    ) -> Result<String, Error> {
        let key = RepositoryInfo::new("github.com", owner, repository_name);
        if let Some(branches) = self.cache.get(&key).await {
            if let Some(default_branch) = &branches.default_branch {
                // tracing::debug!("Exist branch in cache: {default_branch}");
                return Ok(default_branch.to_string());
            }
        }

        let url = format!("https://api.github.com/repos/{owner}/{repository_name}");

        let req = Request::builder()
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "Cloc-Info-App")
            .header("Authorization", USERNAME_TOKEN)
            .uri(&url)
            .body(Body::empty())
            .with_context(|_| CreateRequestSnafu { url: url.clone() })?;

        let https_connector = HttpsConnector::new().unwrap();
        let client = hyper::client::Client::builder().build::<_, hyper::Body>(https_connector);

        let mut response = client
            .request(req)
            .await
            .with_context(|_| SendRequestSnafu { url: url.clone() })?;

        let body = response.body_mut();

        let bytes = hyper::body::to_bytes(body)
            .await
            .with_context(|_| GetResponseBodySnafu { url: url.clone() })?;

        let repository: Value =
            serde_json::from_slice(&bytes).with_context(|_e| DeserializeSnafu {
                bytes: String::from_utf8(bytes.to_vec()).expect("can't vec to string"),
                url,
            })?;

        match repository["default_branch"].as_str() {
            Some(branch) => {
                let branches = if let Some(mut branches) = self.cache.remove(&key).await {
                    tracing::debug!("Update default branch {branch} for {key:?}");
                    branches.default_branch = Some(branch.to_string());
                    branches
                } else {
                    // tracing::debug!("{key:?} doesn't exist in cache or expired. Insert to cache repo and branch {branch}");
                    Branches {
                        default_branch: Some(branch.to_string()),
                        branches: vec![],
                    }
                };
                self.cache
                    .insert(key, branches, Duration::from_secs(60))
                    .await;
                Ok(branch.to_owned())
            }
            None => Err(Error::ExtractDefaultBranchError {
                repo: to_url("github.com", owner, repository_name, ""),
            }),
        }
    }

    pub async fn last_commit_remote(
        &self,
        owner: &str,
        repository_name: &str,
        branch: &str,
    ) -> Result<String, Error> {
        let key = RepositoryInfo::new("github.com", owner, repository_name);
        if let Some(branches) = self.cache.get(&key).await {
            if let Some(branch) = branches.branches.iter().find(|b| b.name == branch) {
                if let Some(commit) = &branch.commit {
                    // tracing::debug!("Exist in cache commit {commit}");
                    return Ok(commit.to_string());
                }
            }
        }

        let url =
            format!("https://api.github.com/repos/{owner}/{repository_name}/commits/{branch}");

        let req = Request::builder()
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "Cloc-Info-App")
            .header("Authorization", USERNAME_TOKEN)
            .uri(&url)
            .body(Body::empty())
            .with_context(|_| CreateRequestSnafu { url: url.clone() })?;

        let https_connector = HttpsConnector::new().unwrap();
        let client = hyper::client::Client::builder().build::<_, hyper::Body>(https_connector);

        let mut response = client
            .request(req)
            .await
            .with_context(|_| SendRequestSnafu { url: url.clone() })?;

        let body = response.body_mut();

        let bytes = hyper::body::to_bytes(body)
            .await
            .with_context(|_| GetResponseBodySnafu { url: url.clone() })?;

        let repository: Value =
            serde_json::from_slice(&bytes).with_context(|_e| DeserializeSnafu {
                bytes: String::from_utf8(bytes.to_vec()).expect("Can't string from bytes"),
                url: url.clone(),
            })?;

        match repository["sha"].as_str() {
            Some(commit) => {
                let branches = if let Some(mut branches) = self.cache.remove(&key).await {
                    let mut values = branches.branches;
                    if let Some(mut value) = values.iter_mut().find(|b| b.name == branch) {
                        value.commit = Some(commit.to_string());
                    } else {
                        let value = BranchValue {
                            name: branch.to_string(),
                            commit: Some(commit.to_string()),
                        };
                        values.push(value);
                    }
                    branches.branches = values;
                    branches
                } else {
                    Branches {
                        default_branch: None,
                        branches: vec![BranchValue {
                            name: branch.to_string(),
                            commit: Some(commit.to_string()),
                        }],
                    }
                };
                self.cache
                    .insert(key, branches, Duration::from_secs(60))
                    .await;
                Ok(commit.to_owned())
            }
            None => Err(Error::ExtractDefaultBranchError {
                repo: to_url("github.com", owner, repository_name, branch),
            }),
        }
    }

    pub async fn remote_branches(
        &self,
        owner: &str,
        repository_name: &str,
    ) -> Result<Vec<BranchInfo>, Error> {
        let url = format!("https://api.github.com/repos/{owner}/{repository_name}/branches");
        let req = Request::builder()
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "Cloc-Info-App")
            .header("Authorization", USERNAME_TOKEN)
            .uri(&url)
            .body(Body::empty())
            .with_context(|_| CreateRequestSnafu { url: url.clone() })?;

        let https_connector = HttpsConnector::new().unwrap();
        let client = hyper::client::Client::builder().build::<_, hyper::Body>(https_connector);

        let mut response = client
            .request(req)
            .await
            .with_context(|_| SendRequestSnafu { url: url.clone() })?;

        let body = response.body_mut();

        let bytes = hyper::body::to_bytes(body)
            .await
            .with_context(|_| GetResponseBodySnafu { url: url.clone() })?;

        let branches_info: Value =
            serde_json::from_slice(&bytes).with_context(|_e| DeserializeSnafu {
                bytes: String::from_utf8(bytes.to_vec()).expect("Cant' vec to string"),
                url: url.clone(),
            })?;

        match branches_info.get("message") {
            Some(v) if v == "Not Found" => return Err(Error::NotFound { url }),
            Some(message) => {
                return Err(Error::RemoteError {
                    url,
                    message: message.to_string(),
                })
            }
            None => {}
        }

        let branches_info: Vec<BranchInfo> =
            serde_json::from_slice(&bytes).with_context(|_e| DeserializeSnafu {
                bytes: String::from_utf8(bytes.to_vec()).expect("Can't vec to string"),
                url,
            })?;

        let key = RepositoryInfo::new("github.com", owner, repository_name);
        let mut branches = Vec::with_capacity(branches_info.len());
        for info in branches_info.iter() {
            let branch = BranchValue {
                name: info.name.clone(),
                commit: Some(info.commit.sha.clone()),
            };
            branches.push(branch);
        }

        let value = if let Some(mut current_branches) = self.cache.remove(&key).await {
            tracing::debug!("Repository {key:?} exist in cache. Update branches in cache");
            current_branches.branches = branches;
            current_branches
        } else {
            // tracing::debug!(
            //     "{key:?} doesn't exist in cache. Insert to cache repo and branches info"
            // );
            Branches {
                default_branch: None,
                branches,
            }
        };
        self.cache.insert(key, value, Duration::from_secs(60)).await;

        Ok(branches_info)
    }
}
