use crate::logic::{
    self,
    info::{to_repository_path, to_url, Status},
    repository::RepositoryProvider,
};
use axum::{
    body::Body,
    extract::{Path, Query, State},
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
        .route("/project/:owner/:repo", get(default_branch_info))
        .route(
            "/project/:owner/:repo/tree/*branch",
            get(branch_commit_info),
        )
        .route("/project/:owner/:repo/branches", get(all_branches_lookup))
        .with_state(provider)
}

pub fn create_general_router(provider: RepositoryProvider) -> Router {
    Router::new()
        .route("/:owner/:repo", get(default_handler))
        .route("/:owner/:repo/src/branch/*branch", get(handler_with_branch))
        .route("/:owner/:repo/tree/*branch", get(handler_with_branch))
        .route("/:owner/:repo/-/tree/*branch", get(handler_with_branch))
        .route("/:owner/:repo/src/*branch", get(handler_with_branch))
        .route("/project/:owner/:repo", get(default_handler))
        .route(
            "/project/:owner/:repo/tree/*branch",
            get(handler_with_branch),
        )
        .with_state(provider)
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn insert_before_head_end(html: &str, snippet: &str) -> String {
    match html.find("</head>") {
        Some(position) => format!("{}{}{}", &html[..position], snippet, &html[position..]),
        None => format!("{html}{snippet}"),
    }
}

/// Replaces the content of the first element delimited by `open` and `close`
/// (for example `<title>` and `</title>`). Falls back to inserting a complete
/// element into the document head when the element is missing.
fn replace_element_content(html: &str, open: &str, close: &str, content: &str) -> String {
    let Some(start) = html.find(open) else {
        return insert_before_head_end(html, &format!("{open}{content}{close}"));
    };
    let content_start = start + open.len();
    let Some(offset) = html[content_start..].find(close) else {
        return html.to_string();
    };
    let content_end = content_start + offset;

    format!(
        "{}{}{}",
        &html[..content_start],
        content,
        &html[content_end..]
    )
}

fn replace_attribute_value(html: &str, prefix: &str, suffix: &str, value: &str) -> Option<String> {
    let start = html.find(prefix)?;
    let value_start = start + prefix.len();
    let offset = html[value_start..].find(suffix)?;
    let value_end = value_start + offset;

    Some(format!(
        "{}{}{}",
        &html[..value_start],
        value,
        &html[value_end..]
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct CountQuery {
    url: Option<String>,
}

/// Turns a pasted repository URL into the report path, e.g.
/// `https://github.com/tokio-rs/axum.git` -> `/github.com/tokio-rs/axum`.
fn repository_path_from_input(input: &str) -> Option<String> {
    let input = input.trim();
    let input = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);
    let input = input.strip_prefix("www.").unwrap_or(input);
    let input = input.split(['?', '#']).next()?;

    let mut segments = input.split('/').filter(|segment| !segment.is_empty());
    let host = segments.next()?;
    let owner = segments.next()?;
    let repository = segments.next()?.trim_end_matches(".git");

    let looks_like_host = host.contains('.') && !host.chars().any(char::is_whitespace);
    if !looks_like_host || owner == ".." || repository.is_empty() || repository == ".." {
        return None;
    }

    Some(format!("/{host}/{owner}/{repository}"))
}

/// Handles the plain HTML form on the home page, so browsers without
/// JavaScript (Lynx, w3m, Links) can still submit a repository URL. The
/// redirect target serves them a text report thanks to the user agent check.
pub async fn count_redirect(Query(query): Query<CountQuery>) -> Response<Body> {
    let Some(path) = query.url.as_deref().and_then(repository_path_from_input) else {
        return (
            StatusCode::BAD_REQUEST,
            [("Content-Type", "text/html; charset=utf-8")],
            "<p>Could not read the repository URL. Use the form https://cloc.info/[git_host]/[git_owner]/[git_repo].</p>",
        )
            .into_response();
    };

    match Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(hyper::header::LOCATION, path)
        .body(Body::empty())
    {
        Ok(response) => response,
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

pub async fn robots() -> Response<Body> {
    (
        [
            ("Cache-Control", "public, max-age=3600"),
            ("Content-Type", "text/plain; charset=utf-8"),
        ],
        crate::statistic::robots_txt(),
    )
        .into_response()
}

fn repository_canonical(host: &str, owner: &str, repository: &str) -> String {
    format!(
        "https://cloc.info/{}",
        to_repository_path(host, owner, repository)
    )
}

/// Adds unique metadata to the shared report template. Without it every
/// repository page would carry the home page title and canonical URL; branch
/// pages are marked `noindex` because they duplicate the repository report.
fn inject_repository_metadata(
    html: &str,
    host: &str,
    owner: &str,
    name: &str,
    branch: Option<&str>,
) -> String {
    let owner = owner.strip_prefix("project/").unwrap_or(owner);
    let repository = name.trim_end_matches(".git");
    let display = format!("{owner}/{repository}");

    let title = match branch {
        Some(branch) => format!("Lines of Code in {display} (branch {branch}) - cloc.info"),
        None => format!("Lines of Code in {display}: SLOC, Comments and COCOMO - cloc.info"),
    };
    let canonical = repository_canonical(host, owner, repository);

    let mut result = replace_element_content(html, "<title>", "</title>", &escape_html(&title));

    result = match replace_attribute_value(
        &result,
        "rel=\"canonical\" href=\"",
        "\"",
        &escape_html(&canonical),
    ) {
        Some(updated) => updated,
        None => insert_before_head_end(
            &result,
            &format!(
                "<link rel=\"canonical\" href=\"{}\">",
                escape_html(&canonical)
            ),
        ),
    };

    if branch.is_some() {
        result = insert_before_head_end(
            &result,
            "<meta name=\"robots\" content=\"noindex, follow\">",
        );
    }

    result
}

fn repository_page(
    host: &str,
    owner: &str,
    name: &str,
    branch: Option<&str>,
) -> Result<Response<Body>, Error> {
    let template = std::fs::read_to_string("dist/info.html").context(TemplatePageIoSnafu)?;
    let html = inject_repository_metadata(&template, host, owner, name, branch);
    Ok((
        [
            ("Cache-Control", "no-cache,private,max-age=0"),
            ("Content-Type", "text/html; charset=utf-8"),
        ],
        html,
    )
        .into_response())
}

// Если нет в БД актуальной информации:
// Начать клонировать параллельно репозиторий.
// Ответить клиенту через 99 секунд 102, а затем через 99 секунд ответ 202
async fn default_handler(
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    state: State<RepositoryProvider>,
    request: Request<Body>, // recommended be last https://docs.rs/axum/latest/axum/extract/index.html#extracting-request-bodies
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
                    Status::Cloned => Response::builder()
                        .status(StatusCode::ACCEPTED)
                        .header("Upgrade", "websocket")
                        .header("Connection", "Upgrade")
                        .body(Body::empty())
                        .context(ResponseSnafu)?,
                    Status::Ready => Response::builder()
                        .status(StatusCode::ACCEPTED)
                        .header("Upgrade", "websocket")
                        .header("Connection", "Upgrade")
                        .body(Body::empty())
                        .context(ResponseSnafu)?,
                    Status::Error(e) => Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(e))
                        .context(ResponseSnafu)?,
                    Status::Previous { date, commit, data } => {
                        let json = serde_json::to_string(&Status::Previous { date, commit, data })
                            .context(SerializeStatusSnafu)?;
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
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, TEXT_PLAIN.essence_str())
                    .body(Body::from("Unsupported If-Match header"))
                    .context(ResponseSnafu)
            }
        }
        None => repository_page(&host, &owner, &name, branch.as_deref()),
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
    match tokio::time::timeout(Duration::from_secs(9), async move {
        Ok(loop {
            let current_status = repository_provider
                .current_status(&key)
                .unwrap_or_else(|| status.clone());

            match current_status {
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
                let message = format!("Your request {} has been received and is currently being processed. Please wait for some time (depending on the size of the repository) and try again.\n", key);
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
            let message = format!("Your request {} has been received and is currently being processed. Please wait for some time (depending on the size of the repository) and try again.\n", unique_name);
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
        || user_agent.contains("Wget")
}

fn extract_user_agent(request: &Request<Body>) -> String {
    match request.headers().get(USER_AGENT) {
        Some(value) => match value.to_str() {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!("Invalid user-agent header: {error}");
                "unknown"
            }
        },
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

    #[snafu(display("Can't load template page: {source}"))]
    TemplatePageIo { source: std::io::Error },

    #[snafu(display("Can't serialize response payload: {source}"))]
    SerializeStatus { source: serde_json::Error },

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

#[cfg(test)]
mod tests {
    use super::extract_user_agent;
    use axum::body::Body;
    use hyper::{
        header::{HeaderValue, USER_AGENT},
        Request,
    };

    use super::{inject_repository_metadata, repository_canonical, repository_path_from_input};

    const TEMPLATE: &str = r#"<html><head><title>Count Lines of Code Online - cloc.info</title><link rel="canonical" href="https://cloc.info"></head><body></body></html>"#;

    #[test]
    fn repository_page_gets_its_own_title_and_canonical() {
        let html = inject_repository_metadata(TEMPLATE, "github.com", "tokio-rs", "axum.git", None);

        assert!(html.contains(
            "<title>Lines of Code in tokio-rs/axum: SLOC, Comments and COCOMO - cloc.info</title>"
        ));
        assert!(html.contains(
            "<link rel=\"canonical\" href=\"https://cloc.info/github.com/tokio-rs/axum\">"
        ));
        assert!(!html.contains("Count Lines of Code Online"));
        assert!(!html.contains("noindex"));
    }

    #[test]
    fn branch_pages_are_noindexed_and_canonical_to_the_repository() {
        let html =
            inject_repository_metadata(TEMPLATE, "github.com", "tokio-rs", "axum", Some("main"));

        assert!(html
            .contains("<title>Lines of Code in tokio-rs/axum (branch main) - cloc.info</title>"));
        assert!(html.contains("<meta name=\"robots\" content=\"noindex, follow\">"));
        assert!(html.contains(
            "<link rel=\"canonical\" href=\"https://cloc.info/github.com/tokio-rs/axum\">"
        ));
    }

    #[test]
    fn canonical_urls_cover_gitflic_and_missing_canonical_tag() {
        assert_eq!(
            repository_canonical("gitflic.ru", "ivan", "project"),
            "https://cloc.info/gitflic.ru/project/ivan/project"
        );

        let without_canonical = inject_repository_metadata(
            "<html><head><title>t</title></head><body></body></html>",
            "gitlab.com",
            "gitlab-org",
            "gitlab",
            None,
        );
        assert!(without_canonical.contains(
            "<link rel=\"canonical\" href=\"https://cloc.info/gitlab.com/gitlab-org/gitlab\">"
        ));
    }

    #[test]
    fn repository_path_accepts_browser_urls() {
        assert_eq!(
            repository_path_from_input("https://github.com/tokio-rs/axum.git").as_deref(),
            Some("/github.com/tokio-rs/axum")
        );
        assert_eq!(
            repository_path_from_input("http://www.gitlab.com/gitlab-org/gitlab/").as_deref(),
            Some("/gitlab.com/gitlab-org/gitlab")
        );
        assert_eq!(
            repository_path_from_input("github.com/torvalds/linux/tree/master").as_deref(),
            Some("/github.com/torvalds/linux")
        );
        assert_eq!(
            repository_path_from_input("  codeberg.org/forgejo/forgejo  ").as_deref(),
            Some("/codeberg.org/forgejo/forgejo")
        );
    }

    #[test]
    fn repository_path_rejects_garbage() {
        assert!(repository_path_from_input("").is_none());
        assert!(repository_path_from_input("github.com/axum").is_none());
        assert!(repository_path_from_input("not a url at all").is_none());
        assert!(repository_path_from_input("https://github.com/../axum").is_none());
        assert!(repository_path_from_input("https://github.com/tokio-rs/.git").is_none());
    }

    #[test]
    fn invalid_user_agent_falls_back_to_unknown() {
        let request = Request::builder()
            .header(USER_AGENT, HeaderValue::from_bytes(&[0xFF]).unwrap())
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_user_agent(&request), "unknown");
    }
}
