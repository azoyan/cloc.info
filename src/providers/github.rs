use crate::{
    cloner::Cloner,
    repository::{
        cache::RepositoryCache,
        info::{to_filename, to_url, BranchInfo},
        utils::{self, count_line_of_code},
    },
    DbId,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use hyper::{Body, Request};
use hyper_openssl::HttpsConnector;
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tokio_postgres::NoTls;

const USERNAME_TOKEN: &str =
    "Basic YXpveWFuOmdocF9IOEVqSXRwMjBOQW9Gc3dYVGI4ektEaktUbkFETlg0TktaNUk=";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Can't deserialize 'github API repo' JSON: {source}"))]
    DeserializeError { source: serde_json::Error },

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
}

pub struct GithubProvider {
    repository_cache: Arc<RwLock<RepositoryCache>>,
    pub connection_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub cloner: Cloner,
}

impl GithubProvider {
    pub fn new(cache_size: usize, connection_pool: Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self {
            repository_cache: Arc::new(RwLock::new(RepositoryCache::new(cache_size))),
            connection_pool,
            cloner: Cloner::new(),
        }
    }

    pub async fn get_with_branch(
        &self,
        owner: &str,
        repository_name: &str,
        branch: Option<&str>,
    ) -> Result<(DbId, Vec<u8>), Error> {
        let start = tokio::time::Instant::now();
        let default_branch = self.default_branch_remote(owner, repository_name).await?;
        let duration = start.elapsed();
        tracing::warn!("Time elapsed in default_branch_remote() is: {:?}", duration);
        
        let branch = match branch {
            Some(branch) => branch,
            None => &default_branch,
        };

        let is_default_branch = branch == default_branch;
        let url = format!("https://github.com/{owner}/{repository_name}");

        let start = tokio::time::Instant::now();
        // Узнаем какой коммит был последний
        let last_commit_remote = self
            .last_commit_remote(owner, repository_name, branch)
            .await?;
        let last_commit_local;
        
        let duration = start.elapsed();
        tracing::warn!("Time elapsed in last_commit_remote() is: {:?}", duration);

        // Новый алгортим
        // 1. Смотрим есть ли в БД
        // 2. Если есть:
        // 2.1 проверяем актуальный ли коммит, если да - возвращаем данные из БД
        // 2.2. если коммит не актуален делаем git pull и пересчитываем, обновляем данные в БД и обнолвяем хранилище на жёстком диске
        // 3. Если отсутствует:
        // 3.1 клонируем репозиторий, вставляем в БД, вставляем в хранилище на жёстком диске.

        // 1. Смотрим есть ли в БД
        let connection = match self.connection_pool.get().await {
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

        let rows = connection
            .query(query, &[&"github.com", &owner, &repository_name, &branch])
            .await
            .with_context(|_e| QuerySnafu {
                query: query.to_owned(),
            })?;

        let repository_path = to_filename("github.com", owner, repository_name, branch);

        let (branch_id, mut cloc) = match rows.get(0) {
            // Есть в БД
            Some(row) => {
                // проверяем актуальность коммита
                let db_last_commit: String = row.get("last_commit_sha");
                let db_branch_name: String = row.get("name");
                let branch_id: DbId = row.get("id");

                if last_commit_remote == db_last_commit && db_branch_name == branch {
                    let scc_output: Vec<u8> = row.get("scc_output");
                    tracing::info!("Current branch and commit are actual. Returning cloc from db");
                    last_commit_local = db_last_commit;
                    (branch_id, scc_output)
                } else {
                    tracing::info!("Current branch '{db_branch_name}' and commit '{db_last_commit}' are not actual '{last_commit_remote}'");

                    let (result, temp_dir) = match self.repository_cache.read().await.get(&url) {
                        Some(temp_dir) => {
                            tracing::info!("Repository {repository_name} cached in disk storage: {repository_path}");
                            // если есть, обновляем репозиторий
                            let result = self.cloner.pull_repository(&url, &repository_path).await;
                            last_commit_local = utils::last_commit_local(&url, &repository_path)
                                .with_context(|_e| LastCommitSnafu)?;
                            (result, temp_dir.clone())
                        }
                        None => {
                            tracing::info!(
                                "Repository {repository_name} is not cached in disk storage"
                            );
                            let temp_dir = Arc::new(
                                TempDir::force_tempdir(&repository_path)
                                    .context(CreateTempDirSnafu)?,
                            );
                            let result = self
                                .cloner
                                .clone_repository(&url, branch, &repository_path)
                                .await;
                            last_commit_local = utils::last_commit_local(&url, &repository_path)
                                .with_context(|_e| LastCommitSnafu)?;
                            (result, temp_dir)
                        }
                    };

                    tracing::info!("Repository {repository_name} cloned state: {:?}", result);
                    let cloc = count_line_of_code(&repository_path, "")
                        .await
                        .context(SccSnafu)?;
                    let repository_id: DbId = rows[0].get("repository_id");
                    tracing::debug!(
                        "INSERT INTO branches VALUES(DEFAULT, {}, {}, {}) ON CONFLICT RETURNING id",
                        repository_id,
                        &branch,
                        &last_commit_local
                    );
                    let upsert_branch = "INSERT INTO branches VALUES(DEFAULT, $1, $2, $3, $4) ON CONFLICT (repository_id, name, last_commit_sha) DO UPDATE SET repository_id = EXCLUDED.repository_id, name = EXCLUDED.name, last_commit_sha = EXCLUDED.last_commit_sha RETURNING id";

                    let rows = connection
                        .query(
                            upsert_branch,
                            &[&repository_id, &branch, &last_commit_local, &cloc],
                        )
                        .await
                        .with_context(|_e| QuerySnafu {
                            query: upsert_branch.to_string(),
                        })?;

                    let branch_id = rows[0].get("id");
                    if is_default_branch {
                        self.repository_cache.write().await.insert(temp_dir);
                    }
                    (branch_id, cloc)
                }
            }
            // Если в БД отсутствует
            None => {
                tracing::warn!("Repository doesn't exist in database and storage cache");
                assert!(self.repository_cache.read().await.get(&url).is_none());

                let temp_dir =
                    Arc::new(TempDir::force_tempdir(&repository_path).context(CreateTempDirSnafu)?);
                // клонируем репозиторий

                let _state = self
                    .cloner
                    .clone_repository(&url, branch, &repository_path)
                    .await;
                last_commit_local = utils::last_commit_local(&url, &repository_path)
                    .with_context(|_e| LastCommitSnafu)?;
                let cloc = count_line_of_code(&repository_path, "")
                    .await
                    .context(SccSnafu)?;

                // вставляем результат в БД
                let upsert_repositories = "insert into repositories values (DEFAULT, $1, $2, $3, $4) ON CONFLICT (hostname, owner, repository_name) DO UPDATE SET hostname=EXCLUDED.hostname, owner=EXCLUDED.owner, repository_name=EXCLUDED.repository_name  RETURNING ID;";
                let rows = connection
                    .query(
                        upsert_repositories,
                        &[&"github.com", &owner, &repository_name, &default_branch],
                    )
                    .await
                    .with_context(|_e| QuerySnafu {
                        query: upsert_repositories.to_string(),
                    })?;

                let repository_id: DbId = rows[0].get("id");

                let insert_branch =
                    "INSERT INTO branches VALUES(DEFAULT, $1, $2, $3, $4) RETURNING id";

                let rows = connection
                    .query(
                        insert_branch,
                        &[&repository_id, &branch, &last_commit_local, &cloc],
                    )
                    .await
                    .with_context(|_e| QuerySnafu {
                        query: insert_branch.to_string(),
                    })?;

                let branch_id = rows[0].get("id");
                // добовалем в хранилище на жёстком диске
                tracing::info!("Clone done. Returning scc_output");
                if is_default_branch {
                    self.repository_cache.write().await.insert(temp_dir);
                }
                (branch_id, cloc)
            }
        };
        let bc = format!("URL: {url}\nBranch: {branch}\nCommit: {last_commit_local}\n");
        cloc.extend(bc.as_bytes());

        Ok((branch_id, cloc))
    }

    pub async fn default_branch_remote(
        &self,
        owner: &str,
        repository_name: &str,
    ) -> Result<String, Error> {
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
            .with_context(|_| GetResponseBodySnafu { url })?;

        let repository: Value = serde_json::from_slice(&bytes).context(DeserializeSnafu)?;

        match repository["default_branch"].as_str() {
            Some(branch) => Ok(branch.to_owned()),
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
            .with_context(|_| GetResponseBodySnafu { url })?;

        let repository: Value = serde_json::from_slice(&bytes).context(DeserializeSnafu)?;

        match repository["sha"].as_str() {
            Some(branch) => Ok(branch.to_owned()),
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
            .with_context(|_| GetResponseBodySnafu { url })?;

        let branches: Vec<BranchInfo> = serde_json::from_slice(&bytes).context(DeserializeSnafu)?;

        Ok(branches)
    }
}
