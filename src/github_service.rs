use crate::{
    providers::github_provider::{self, GithubProvider},
    repository::utils,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use hyper::{
    header::{self, CONTENT_TYPE, USER_AGENT},
    Body, Request, StatusCode,
};
use mime_guess::mime::{APPLICATION_JSON, TEXT_PLAIN};
use serde_json::json;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn create_api_router(provider: Arc<RwLock<GithubProvider>>) -> Router<(), Body> {
    Router::new()
        .route("/:owner/:repo", get(default_branch_info))
        .route("/:owner/:repo/tree/*branch", get(branch_commit_info))
        .route("/:owner/:repo/-/tree/*branch", get(branch_commit_info))
        .route("/:owner/:repo/src/*branch", get(branch_commit_info))
        .route("/:owner/:repo/branches", get(all_branches_lookup))
        .with_state(provider)
}

pub fn create_router(provider: Arc<RwLock<GithubProvider>>) -> Router<(), Body> {
    let router = Router::new()
        .route("/", get(default_handler))
        .route("/tree/*branch", get(handler_with_branch))
        .route("/-/tree/*branch", get(handler_with_branch))
        .route("/src/*branch", get(handler_with_branch));

    Router::new()
        .nest("/:owner/:repo", router)
        .with_state(provider)
}

fn static_page() -> Result<Response<Body>, Error> {
    let file = std::fs::File::open("static/info.html").unwrap();
    let mut reader = std::io::BufReader::new(file);

    let mut buffer = vec![];

    std::io::Read::read_to_end(&mut reader, &mut buffer).unwrap();
    Response::builder()
        .header("Cache-Control", "no-cache,private,max-age=0") // TODO, maybe rewrite AddHeader
        .body(Body::from(buffer))
        .context(StaticPageSnafu)
}

async fn raw_content(
    provider: Arc<RwLock<GithubProvider>>,
    host: &str,
    owner: &str,
    repository_name: &str,
    branch: Option<&str>,
    user_agent: &str,
) -> Result<Response<Body>, Error> {
    let result = {
        let provider_guard = provider.read().await;
        provider_guard
            .get_with_branch(host, owner, repository_name, branch, user_agent)
            .await
    };

    let (response, _branch_id) = match result {
        Ok((branch_id, scc_output)) => (
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from(scc_output))
                .context(ResponseSnafu)?,
            branch_id,
        ),
        Err(e) => {
            if let github_provider::Error::InProgress { url } = e {
                tracing::warn!("Repository {url} dowonloading already in progress");
                (
                    Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .header("Content-Type", "text/plain")
                        .body(Body::empty())
                        .context(ResponseSnafu)?,
                    0,
                )
            } else {
                tracing::error!("{}", e);
                (
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "text/plain")
                        .body(Body::from(e.to_string()))
                        .context(ResponseSnafu)?,
                    0,
                )
            }
        }
    };
    Ok(response)
}

async fn default_handler(
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    Extension(provider): Extension<Arc<RwLock<GithubProvider>>>,
    request: Request<Body>, // recomended be last https://docs.rs/axum/latest/axum/extract/index.html#extracting-request-bodies
) -> Result<Response<Body>, Error> {
    tracing::debug!("Default Handler {:?}, host: {host}", request);
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    handle_request(&host, &owner, &repository_name, None, provider, request).await
}

async fn handler_with_branch(
    Path((host, owner, mut repository_name, branch_name)): Path<(String, String, String, String)>,
    Extension(provider): Extension<Arc<RwLock<GithubProvider>>>,
    request: Request<Body>, // recomended be last https://docs.rs/axum/latest/axum/extract/index.html#extracting-request-bodies
) -> Result<Response<Body>, Error> {
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let branch_name = if host == "codeberg.org" {
        tracing::warn!("{branch_name}");
        branch_name.trim_start_matches("/branch")
    } else {
        &branch_name
    };
    let branch = &branch_name[1..];
    let branch = branch.trim_end_matches('/');
    tracing::debug!("Handler with branch {:?}, branch: {branch}", request,);
    handle_request(
        &host,
        &owner,
        &repository_name,
        Some(branch),
        provider,
        request,
    )
    .await
}

// Cloudflare ограничения на выполнение запроса 100 секунд
// Запросить ресурс, если доступен выдать ответ
// если не доступен выдать WS

async fn handle_request(
    host: &str,
    owner: &str,
    repository_name: &str,
    branch: Option<&str>,
    provider: Arc<RwLock<GithubProvider>>,
    request: Request<Body>,
) -> Result<Response<Body>, Error> {
    let user_agent = match request.headers().get(USER_AGENT) {
        Some(value) => value.to_str().unwrap_or("not valid utf-8"),
        None => "unknown",
    };

    if user_agent.contains("Lynx")
        || user_agent.contains("w3m")
        || user_agent.contains("Links")
        // Netrik User agent
        || user_agent.contains("Not mandatory")
        || user_agent.contains("curl")
    {
        tracing::info!("Terminal browser: {:?}", user_agent);
        let response = raw_content(
            provider.clone(),
            host,
            owner,
            repository_name,
            branch,
            user_agent,
        )
        .await
        .unwrap();

        return Ok(response);
    }

    match request.headers().get(header::IF_MATCH) {
        Some(value) => {
            let value = match value.to_str() {
                Ok(v) => v,
                Err(e) => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(e.to_string()))
                        .context(ResponseSnafu)
                }
            };

            if value.contains("cloc") {
                let response = raw_content(
                    provider.clone(),
                    host,
                    owner,
                    repository_name,
                    branch,
                    user_agent,
                )
                .await?;
                tracing::info!("Response Ready for {host}/{owner}/{repository_name}/{branch:?}");
                Ok(response)
            } else {
                // ws
                Response::builder()
                    .body(Body::empty())
                    .context(ResponseSnafu)
            }
        }
        None => static_page(),
    }
}

async fn all_branches_lookup(
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    State(provider): State<Arc<RwLock<GithubProvider>>>,
    _request: Request<Body>,
) -> Result<Response<Body>, Error> {
    tracing::warn!("all_branches_lookup() host: {host}, owner: {owner}, repo: {repository_name}");
    let provider_guard = provider.read().await;
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let branches_info = provider_guard
        .remote_branches(&host, &owner, &repository_name)
        .await
        .with_context(|_e| GithubProviderSnafu)?;

    match serde_json::to_string(&branches_info) {
        Ok(branches) => Response::builder()
            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
            .body(Body::from(branches))
            .context(ResponseSnafu),
        Err(e) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(CONTENT_TYPE, TEXT_PLAIN.essence_str())
            .body(Body::from(e.to_string()))
            .context(ResponseSnafu),
    }
}

async fn default_branch_info(
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    State(provider): State<Arc<RwLock<GithubProvider>>>,
    _request: Request<Body>,
) -> Result<Response<Body>, Error> {
    tracing::warn!("default_branch_info() host: {host}, owner: {owner}, repo: {repository_name}");
    let provider_guard = provider.read().await;
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let default_branch = provider_guard
        .default_branch_remote(&host, &owner, &repository_name)
        .await;

    match default_branch {
        Ok(default_branch) => {
            let json = json!({ "default_branch": default_branch });
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(json.to_string()))
                .context(ResponseSnafu)
        }
        Err(e) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(CONTENT_TYPE, TEXT_PLAIN.essence_str())
            .body(Body::from(e.to_string()))
            .context(ResponseSnafu),
    }
}

async fn branch_commit_info(
    Path((host, owner, mut repository_name, branch)): Path<(String, String, String, String)>,
    State(provider): State<Arc<RwLock<GithubProvider>>>,
) -> Result<Response<Body>, Error> {
    tracing::warn!("branch_commit_info() host: {host}, owner: {owner}, repo: {repository_name}, branch: {branch}");
    let provider_guard = provider.read().await;
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let branch = if host == "codeberg.org" {
        branch.trim_start_matches("/branch")
    } else {
        &branch
    };
    let commit = provider_guard
        .last_commit_remote(&host, &owner, &repository_name, branch)
        .await;
    match commit {
        Ok(commit) => {
            let json = json!({ "commit": commit });
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(json.to_string()))
                .context(ResponseSnafu)
        }
        Err(e) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
            .body(Body::from(e.to_string()))
            .context(ResponseSnafu),
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Can't create StaticPage: {source}"))]
    StaticPage { source: axum::http::Error },

    #[snafu(display("Can't create respnose: {source}"))]
    ResponseError { source: axum::http::Error },

    #[snafu(display("Template page not found"))]
    TemplatePage,

    #[snafu(display("Branch '{wrong_branch}' is note exist"))]
    WrongBranch { wrong_branch: String },

    #[snafu(display("Unrecoginzed If-Match header"))]
    IfMatchError,

    #[snafu(display("Error at cloning repository or scc: {source}"))]
    DownloaderError { source: utils::Error },

    #[snafu(display("Error at github provider: {source}"))]
    GithubProviderError {
        source: crate::providers::github_provider::Error,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let msg = self.to_string();
        let status = StatusCode::INTERNAL_SERVER_ERROR;

        tracing::error!("{msg}");

        let body = Json(json!({
            "error": msg,
        }));

        (status, body).into_response()
    }
}
