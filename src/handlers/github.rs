use crate::repository::{
    cache::RepositoryCache,
    info::{to_url, BranchInfo, RepositoryInfo},
    utils::{self, clone_branch, count_line_of_code, pull},
};
use actix_web::{
    error::{self, PayloadError},
    get,
    http::{
        header::{CacheControl, CacheDirective, ContentType},
        StatusCode,
    },
    web, HttpRequest, HttpResponse, ResponseError,
};
use awc::error::{JsonPayloadError, SendRequestError};
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use tokio::sync::RwLock;

// recommendations from https://docs.github.com/en/rest/reference/branches
const HEADER: (&str, &str) = ("Accept", "application/vnd.github.v3+json");

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("JSON deserialization error: {source}"))]
    JsonError { source: JsonPayloadError },

    #[snafu(display("Can't receive response body: {source}"))]
    BodyError { source: PayloadError },

    #[snafu(display("Can't deserialize 'github API repo' JSON: {source}"))]
    DeserializeError { source: serde_json::Error },

    #[snafu(display("Can't extract default branch for repository {repo}"))]
    ExtractDefaultBranchError { repo: String },

    #[snafu(display("Template page not found"))]
    TemplatePage,

    #[snafu(display("Can't send request about repository '{url}': {source}"))]
    BranchInfoRequestError {
        url: String,
        source: SendRequestError,
    },

    #[snafu(display("Branch '{wrong_branch}' is note exist"))]
    WrongBranch { wrong_branch: String },

    #[snafu(display("Unrecoginzed If-Match header"))]
    IfMatchError,

    #[snafu(display("Error at cloning repository or scc: {source}"))]
    DownloaderError { source: utils::Error },
}

#[get("/")]
async fn handle_github_dummy(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    _provider: web::Data<RwLock<GithubProvider>>,
) -> HttpResponse {
    let (_owner, _repository_name) = path.into_inner();
    println!("Request: {request:?}");
    HttpResponse::Ok().content_type("plain/text").body("debil")
}

/// 1. Проверка URL: Какие есть ветки? curl -s https://api.github.com/repos/azoyan/talua/branches | jq --raw-output | grep "name"
/// 2. Смотрим в кеш, есть ли там нужная страница?
/// 3. Если в кеше есть нужная страница проверяем актуальная ли она: смотрим remote на последний коммит в этой ветке:  
/// 4. Если комиты совпадают отдаём страницу из кеша  
/// 5. Если комиты не совпадают выкачиваем репозиторий и обновляем кеш

#[get("/{owner}/{repo}")]
async fn handle_github(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    provider: web::Data<RwLock<GithubProvider>>,
) -> HttpResponse {
    let (owner, repository_name) = path.into_inner();
    println!("Request: {request:?}");

    fn static_page() -> HttpResponse {
        if let Ok(contents) = std::fs::read_to_string("static/info.html") {
            let cache_control = CacheControl(vec![
                CacheDirective::NoCache,
                CacheDirective::Private,
                CacheDirective::MaxAge(0u32),
            ]);
            let mut response = HttpResponse::Ok();
            response.insert_header(cache_control);

            response.body(contents)
        } else {
            HttpResponse::InternalServerError().finish()
        }
    }

    let raw_content = || async {
        let mut provider = provider.write().await;

        let info = provider.get(&owner, &repository_name).await;

        match info {
            Ok(info) => HttpResponse::Ok()
                .content_type("text/plain")
                .body(info.scc_output),
            Err(e) => HttpResponse::InternalServerError()
                // .status(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/plain")
                .body(e.to_string()),
        }
    };

    match request.headers().get(actix_web::http::header::IF_MATCH) {
        Some(value) => {
            let value = match value.to_str() {
                Ok(v) => v,
                Err(_e) => return Error::IfMatchError.error_response(),
            };

            if value.contains("cloc") {
                raw_content().await
            } else {
                static_page()
            }
        }
        None => static_page(),
    }
}

#[get("/github.com/{owner}/{repo}/branches")]
async fn get_branches(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    provider: web::Data<RwLock<GithubProvider>>,
) -> HttpResponse {
    let (owner, repository_name) = path.into_inner();
    println!("Request: {request:?}");

    let provider = provider.read().await;

    let branches = provider.remote_branches(&owner, &repository_name).await;

    match branches {
        Ok(info) => match serde_json::to_string(&info) {
            Ok(body) => {
                log::info!("body = {body}");
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            Err(e) => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(e.to_string()),
    }
}

#[get("/{user}/{name}/tree/{branch}")]
async fn handle_github_branch(
    _request: HttpRequest,
    path: web::Path<(String, String, String)>,
    provider: web::Data<RwLock<GithubProvider>>,
) -> HttpResponse {
    let (owner, repository_name, branch) = path.into_inner();

    let mut provider = provider.write().await;

    let info = provider
        .get_with_branch(&owner, &repository_name, &branch)
        .await;

    match info {
        Ok(info) => {
            let mut branch = format!("{}\n",info.branch).into_bytes();
            let mut commit = format!("{}\n",info.branch).into_bytes();
            
            let mut output = vec![];
            output.append(&mut branch);
            output.append(&mut commit);
            output.extend(&info.scc_output);

            HttpResponse::Ok()
                .content_type("text/plain")
                .body(info.scc_output)
        }
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(e.to_string()),
    }
}

unsafe impl Send for GithubProvider {}
unsafe impl Sync for GithubProvider {}
pub struct GithubProvider {
    repository_cache: RepositoryCache,
    client: awc::Client,
}

impl GithubProvider {
    pub fn new() -> Self {
        Self {
            repository_cache: RepositoryCache::new(10000),
            client: awc::Client::new(),
        }
    }

    pub async fn get(
        &mut self,
        owner: &str,
        repository_name: &str,
    ) -> Result<RepositoryInfo, Error> {
        // Узнаем когда ветка по умолчанию
        let default_branch = self.default_branch_remote(owner, repository_name).await?;

        self.get_with_branch(owner, repository_name, &default_branch)
            .await
    }

    pub async fn get_with_branch(
        &mut self,
        owner: &str,
        repository_name: &str,
        branch: &str,
    ) -> Result<RepositoryInfo, Error> {
        let url = to_url("github.com", owner, repository_name, branch);
        // Узнаем какой коммит был последний
        let last_commit_remote = self
            .last_commit_remote(owner, repository_name, branch)
            .await?;

        // Смотрим есть ли уже этот репозиторий в ветке
        let cache_lookup = self.repository_cache.get(&url);
        let (last_commit, local_dir, size) = if let Some(repository_info) = cache_lookup {
            log::info!(
                "Lookup at cache: last_commit_remote = {}, last_commit_cache = {}",
                last_commit_remote,
                repository_info.last_commit
            );
            // Если во внешнем репозитории такой же последний коммит как в кеше
            if repository_info.last_commit == last_commit_remote {
                // отдаём инофрмацию из кеша
                return Ok(repository_info.to_owned());
            } else {
                log::info!("Pull branch: {url}");
                // обновляем репозиторий до последнего коммита
                let (last_commit, size) = pull(
                    repository_name,
                    &repository_info.local_dir.path,
                    &repository_info.branch,
                )
                .await
                .context(DownloaderSnafu)?;

                (last_commit, repository_info.local_dir.clone(), size)
            }
        } else {
            // Клонируем ветку
            log::info!("Clone branch: {url}");

            clone_branch("github.com", owner, repository_name, branch)
                .await
                .context(DownloaderSnafu)?
        };
        let scc_output = count_line_of_code(&local_dir.path, "")
            .await
            .context(DownloaderSnafu)?;

        let repository_info = RepositoryInfo {
            hostname: "github.com".to_string(),
            owner: owner.to_string(),
            repository_name: repository_name.to_string(),
            branch: branch.to_string(),
            last_commit,
            local_dir,
            size,
            scc_output,
        };

        self.repository_cache.insert(repository_info.clone());

        Ok(repository_info)
    }

    pub async fn remote_branches(
        &self,
        owner: &str,
        repository_name: &str,
    ) -> Result<Vec<BranchInfo>, Error> {
        let url = format!("https://api.github.com/repos/{owner}/{repository_name}/branches");
        println!("URL = {url}");
        let mut response = self
            .client
            .get(url)
            .insert_header(HEADER)
            .insert_header(("User-Agent", "Cloc-Info-App"))
            .send()
            .await
            .context(BranchInfoRequestSnafu {
                url: to_url("github.com", owner, repository_name, ""),
            })?;

        response.json::<Vec<BranchInfo>>().await.context(JsonSnafu)
    }

    pub async fn check_remote_branch(
        &self,
        owner: &str,
        repository_name: &str,
        branch: &str,
    ) -> Result<(), Error> {
        let branches = self.remote_branches(owner, repository_name).await?;

        match branches.iter().find(|info| info.name == branch) {
            Some(_branch) => Ok(()),
            None => Err(Error::WrongBranch {
                wrong_branch: branch.to_owned(),
            }),
        }
    }

    pub async fn default_branch_remote(
        &self,
        owner: &str,
        repository_name: &str,
    ) -> Result<String, Error> {
        let url = format!("https://api.github.com/repos/{owner}/{repository_name}");
        let mut response = self
            .client
            .get(url)
            .insert_header(HEADER)
            .insert_header(("User-Agent", "Cloc-Info-App"))
            .send()
            .await
            .context(BranchInfoRequestSnafu {
                url: to_url("github.com", owner, repository_name, ""),
            })?;

        let bytes = response.body().await.context(BodySnafu)?;
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
        println!("URL = {url}");
        let mut response = self
            .client
            .get(url)
            .insert_header(HEADER)
            .insert_header(("User-Agent", "Cloc-Info-App"))
            .send()
            .await
            .context(BranchInfoRequestSnafu {
                url: to_url("github.com", owner, repository_name, branch),
            })?;

        let bytes = response.body().await.context(BodySnafu)?;
        let repository: Value = serde_json::from_slice(&bytes).context(DeserializeSnafu)?;
        match repository["sha"].as_str() {
            Some(branch) => Ok(branch.to_owned()),
            None => Err(Error::ExtractDefaultBranchError {
                repo: to_url("github.com", owner, repository_name, branch),
            }),
        }
    }
}

impl Default for GithubProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match self {
            Error::JsonError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BodyError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DeserializeError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ExtractDefaultBranchError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::TemplatePage => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BranchInfoRequestError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::WrongBranch { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DownloaderError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::IfMatchError => StatusCode::BAD_REQUEST,
        }
    }
}
