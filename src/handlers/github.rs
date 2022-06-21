use std::{io::Read, sync::RwLock};

use crate::repository::{
    cache::RepositoryCache,
    downloader::{self, cloc_branch},
    info::{to_url, BranchInfo, RepositoryInfo},
};
use actix_web::{
    error::PayloadError,
    get,
    http::header::{CacheControl, CacheDirective},
    web, HttpRequest, HttpResponse,
};
use awc::error::{JsonPayloadError, SendRequestError};
use serde_json::Value;
use snafu::{ResultExt, Snafu};

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

    #[snafu(display("Can't send request about repository '{url}': {source}"))]
    BranchInfoRequestError {
        url: String,
        source: SendRequestError,
    },

    #[snafu(display("Branch '{wrong_branch}' is note exist"))]
    WrongBranch { wrong_branch: String },

    #[snafu(display("Error at cloning repository or scc: {source}"))]
    DownloaderError { source: downloader::Error },
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
        let info = actix_files::NamedFile::open("static/info.html").unwrap();

        let cache_control = CacheControl(vec![
            CacheDirective::NoCache,
            CacheDirective::Private,
            CacheDirective::MaxAge(0u32),
        ]);
        let mut response = HttpResponse::Ok();
        response.insert_header(cache_control);

        let mut contents = String::new();
        info.file().read_to_string(&mut contents).unwrap();

        return response.body(contents);
    }

    let raw_content =  || async {
        let mut provider = provider.write().unwrap();

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
            let value = value.to_str().unwrap();
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

    let provider = provider.read().unwrap();

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

    let mut provider = provider.write().unwrap();

    let info = provider
        .get_with_branch(&owner, &repository_name, &branch)
        .await;

    match info {
        Ok(info) => HttpResponse::Ok()
            .content_type("text/plain")
            .body(info.scc_output),
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
        if let Some(repository_info) = cache_lookup {
            log::info!(
                "Lookup at cache: last_commit_remote = {}, last_commit_cache = {}",
                last_commit_remote,
                repository_info.last_commit
            );
            // Если во внешнем репозитории такой же последний коммит как в кеше
            if repository_info.last_commit == last_commit_remote {
                // отдаём инофрмацию из кеша
                return Ok(repository_info.to_owned());
            }
        };
        // Качаем ветку
        log::info!("Clone branch: {url}");
        let repository_info = cloc_branch("github.com", owner, repository_name, branch)
            .await
            .context(DownloaderSnafu)?;

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
        println!("{}", std::str::from_utf8(&bytes).unwrap());
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
        println!("{}", std::str::from_utf8(&bytes).unwrap());
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
