use crate::logic::{
    self,
    info::{to_url, Status},
    repository::RepositoryProvider,
};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use hyper::{
    header::{self, CONTENT_TYPE, USER_AGENT},
    Request, StatusCode,
};
use mime_guess::mime::{APPLICATION_JSON, TEXT_PLAIN};
use serde_json::json;
use snafu::{ResultExt, Snafu};
use std::time::Duration;

pub fn create_api_router(provider: RepositoryProvider) -> Router {
    Router::new()
        .route("/:owner/:repo", get(default_branch_info))
        .route("/:owner/:repo/src/branch/*branch", get(branch_commit_info))
        .route("/:owner/:repo/tree/*branch", get(branch_commit_info))
        .route("/:owner/:repo/-/tree/*branch", get(branch_commit_info))
        .route("/:owner/:repo/src/*branch", get(branch_commit_info))
        .route("/:owner/:repo/branches", get(all_branches_lookup))
        .with_state(provider)
}

pub fn create_general_router(provider: RepositoryProvider) -> Router {
    let router = Router::new()
        .route("/", get(default_handler))
        .route("/src/branch/*branch", get(handler_with_branch))
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

// Если нет в БД актуальной информации:
// Начать клонировать параллельно репозиторий.
// Ответить клиенту через 99 секунд 102, а затем через 99 секунд ответ 202
async fn default_handler(
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    state: State<RepositoryProvider>,
    request: Request<Body>, // recomended be last https://docs.rs/axum/latest/axum/extract/index.html#extracting-request-bodies
) -> Result<Response<Body>, Error> {
    tracing::debug!("Default Handler {:?}, host: {host}", request);
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    handle_repository(host, owner, repository_name, None, state, request).await
}

async fn handler_with_branch(
    Path((host, owner, mut repository_name, branch_name)): Path<(String, String, String, String)>,
    state: State<RepositoryProvider>,
    request: Request<Body>, // recommended be last https://docs.rs/axum/latest/axum/extract/index.html#extracting-request-bodies
) -> Result<Response<Body>, Error> {
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    // let branch_name = if host == "codeberg.org" {
    //     tracing::warn!("{branch_name}");
    //     branch_name.trim_start_matches("/branch")
    // } else {
    //     &branch_name
    // };
    let branch = branch_name
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();

    tracing::debug!("Handler with branch {:?}, branch: {branch}", request);
    handle_repository(host, owner, repository_name, Some(branch), state, request).await
}

// Cloudflare ограничения на выполнение запроса 100 секунд
// Запросить ресурс, если доступен выдать ответ
// если не доступен выдать WS
async fn handle_repository(
    host: String,
    owner: String,
    name: String,
    branch: Option<String>,
    State(provider): State<RepositoryProvider>,
    request: Request<Body>,
) -> Result<Response<Body>, Error> {
    let user_agent = extract_user_agent(&request);

    if is_terminal_browser(&user_agent) {
        terminal_browser(host, owner, name, branch, user_agent, provider).await
    } else {
        regular(host, owner, name, branch, user_agent, provider, request).await
    }
}

async fn regular(
    host: String,
    owner: String,
    name: String,
    branch: Option<String>,
    user_agent: String,
    state: RepositoryProvider,
    request: Request<Body>,
) -> Result<Response<Body>, Error> {
    match request.headers().get(header::IF_MATCH) {
        Some(value) => {
            let value = match value.to_str() {
                Ok(v) => v,
                Err(e) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(e.to_string()))
                        .context(ResponseSnafu)
                }
            };

            if value.contains("cloc") {
                let (unique_name, status) = state
                    .request_info(host, owner, name, branch, user_agent)
                    .await
                    .context(GithubProviderSnafu)?;
                tracing::warn!("After request_info {unique_name}, {}", status);

                let response = match status {
                    Status::Done(scc_output) => Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, TEXT_PLAIN.essence_str())
                        .body(Body::from(scc_output))
                        .context(ResponseSnafu)?,
                    Status::InProgress(_s) => Response::builder()
                        .status(StatusCode::ACCEPTED)
                        .header("Upgrade", "websocket")
                        .header("Connection", "Upgrade")
                        .body(Body::empty())
                        .context(ResponseSnafu)?,
                    Status::Cloned => unreachable!(),
                    Status::Ready => Response::builder()
                        .status(StatusCode::ACCEPTED)
                        .header("Upgrade", "websocket")
                        .header("Connection", "Upgrade")
                        .body(Body::empty())
                        .context(ResponseSnafu)?,
                    Status::Error(e) => Response::builder()
                        .status(StatusCode::ACCEPTED)
                        .body(Body::from(e))
                        .context(ResponseSnafu)?,
                    Status::Previous { date, commit, data } => {
                        let json = serde_json::to_string(&Status::Previous { date, commit, data })
                            .unwrap();
                        // tracing::warn!("previous: {}", json);
                        Response::builder()
                            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                            .status(StatusCode::PARTIAL_CONTENT)
                            .body(Body::from(json))
                            .context(ResponseSnafu)?
                    }
                };
                Ok(response)
            } else {
                Response::builder() // ws
                    .body(Body::empty())
                    .context(ResponseSnafu)
            }
        }
        None => static_page(),
    }
}

async fn terminal_browser(
    host: String,
    owner: String,
    name: String,
    branch: Option<String>,
    user_agent: String,
    repository_provider: RepositoryProvider,
) -> Result<Response<Body>, Error> {
    tracing::info!("Terminal browser: {:?}", user_agent);
    let (unique_name, status) = repository_provider
        .request_info(host, owner, name, branch, user_agent)
        .await
        .context(GithubProviderSnafu)?;

    tracing::debug!("After request_info for {unique_name}: {}", status);

    let mut counter = 5;
    let key = unique_name.clone();
    match tokio::time::timeout(Duration::from_secs(9), async move  {
        Ok(loop {
            // let status = repository_provider.current_status(&key);
            match status {
                Status::Done(scc_output) => {
                    break Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, "text/plain")
                        .body(Body::from(scc_output))
                        .context(ResponseSnafu)?;
                }
                Status::InProgress(_) => {}
                Status::Cloned => {}
                Status::Ready => {}
                Status::Error(e) => {
                    break Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(e))
                        .context(ResponseSnafu)?;
                }
                Status::Previous {date, commit, data} => break Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(CONTENT_TYPE, TEXT_PLAIN.essence_str())
                .body(Body::from(message_for_previous_result(date, commit, data)))
                .context(ResponseSnafu)?,
            };
            if counter == 0 {
                let message = format!("Your request {} has been received and we are currently processing it. Please wait for a 5 minutes and try again.\n", key);
                break Response::builder()
                    .status(StatusCode::ACCEPTED)
                    .body(Body::from(message))
                    .context(ResponseSnafu)?;
            }
            counter -= 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
        })
    }).await {
        Ok(response) => response,
        Err(_elapsed) => {
            let message = format!("Your request {} has been received and we are currently processing it. Please wait for a 5 minutes and try again.\n", unique_name);
                 Response::builder()
                    .status(StatusCode::ACCEPTED)
                    .body(Body::from(message))
                    .context(ResponseSnafu)
        },
    }
}

fn is_terminal_browser(user_agent: &str) -> bool {
    user_agent.contains("Lynx")
        || user_agent.contains("w3m")
        || user_agent.contains("Links")
        || user_agent.contains("Not mandatory") // Netrik User agent
        || user_agent.contains("curl")
}

fn extract_user_agent(request: &Request<Body>) -> String {
    match request.headers().get(USER_AGENT) {
        Some(value) => value.to_str().expect("not valid utf-8"),
        None => "unknown",
    }
    .to_string()
}
async fn all_branches_lookup(
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    State(provider): State<RepositoryProvider>,
    _request: Request<Body>,
) -> Result<Response<Body>, Error> {
    tracing::warn!("all_branches_lookup() host: {host}, owner: {owner}, repo: {repository_name}");
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let url = to_url(&host, &owner, &repository_name);
    let branches_info = provider
        .remote_branches(&url)
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
    State(provider): State<RepositoryProvider>,
    _request: Request<Body>,
) -> Result<Response<Body>, Error> {
    tracing::debug!("default_branch_info() host: {host}, owner: {owner}, repo: {repository_name}");
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let default_branch = provider
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
    State(provider): State<RepositoryProvider>,
) -> Result<Response<Body>, Error> {
    tracing::info!("branch_commit_info() host: {host}, owner: {owner}, repo: {repository_name}, branch: {branch}");
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let branch = if host == "codeberg.org" {
        branch.trim_start_matches("/branch")
    } else {
        &branch
    };
    let commit = provider
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

    #[snafu(display("Can't create response: {source}"))]
    ResponseError { source: axum::http::Error },

    #[snafu(display("Template page not found"))]
    TemplatePage,

    #[snafu(display("Branch '{wrong_branch}' is note exist"))]
    WrongBranch { wrong_branch: String },

    #[snafu(display("Unrecognized If-Match header"))]
    IfMatchError,

    #[snafu(display("Error at cloning repository or scc: {source}"))]
    DownloaderError { source: logic::Error },

    #[snafu(display("Error at github provider: {source}"))]
    GithubProviderError { source: logic::Error },
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

fn message_for_previous_result(date: DateTime<Utc>, commit: String, data: Vec<u8>) -> Vec<u8> {
    let warn = format!(
        "The information about the repository provided below is accurate as of {} and applies to commit {}.\n",
        date.to_rfc3339(),
        commit
    );

    let reminder = b"Currently, the repository is being downloaded and updated. Please check back in 5 minutes.\n";

    [warn.as_bytes(), data.as_slice(), reminder].concat()
}
