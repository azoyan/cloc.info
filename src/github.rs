use crate::{providers::github::GithubProvider, repository::utils, DbId};
use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use axum_macros::debug_handler;

use hyper::{
    header::{self, CONTENT_TYPE, USER_AGENT},
    Body, Request, StatusCode,
};
use mime_guess::mime::{APPLICATION_JSON, TEXT_PLAIN};
use serde_json::json;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn create_router(github_provider: GithubProvider) -> Router<Body> {
    let cloner = github_provider.cloner.clone();
    let provider = Arc::new(RwLock::new(github_provider));

    let router = Router::new()
        .route("/", get(default_handler))
        .route(
            "/ws/:hostname/:owner/:repository_name",
            axum::routing::get(crate::websocket::handler_ws),
        )
        .route("/branches", get(all_branches_lookup));

    Router::new()
        .nest("/:owner/:repo", router)
        .layer(Extension(provider))
        .layer(Extension(cloner))
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
    owner: &str,
    repository_name: &str,
) -> Result<(Response<Body>, DbId), Error> {
    let result = {
        let provider_guard = provider.read().await;
        provider_guard.get(owner, repository_name).await
    };

    let (response, branch_id) = match result {
        Ok((branch_id, scc_output)) => (
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from(scc_output))
                .context(ResponseSnafu)?,
            branch_id,
        ),
        Err(e) => (
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain")
                .body(Body::from(e.to_string()))
                .context(ResponseSnafu)?,
            0,
        ),
    };
    Ok((response, branch_id))
}

#[debug_handler]
async fn default_handler(
    Path((owner, repository_name)): Path<(String, String)>,
    Extension(provider): Extension<Arc<RwLock<GithubProvider>>>,
    request: Request<Body>, // recomended be last https://docs.rs/axum/latest/axum/extract/index.html#extracting-request-bodies
) -> Result<Response<Body>, Error> {
    tracing::debug!("default handler {:?}", request);

    let user_agent = match request.headers().get(USER_AGENT) {
        Some(value) => value.to_str().unwrap_or("not valid utf-8"),
        None => "unknown",
    };

    if user_agent.contains("Lynx")
        || user_agent.contains("w3m")
        || user_agent.contains("Links")
        // Netrik User agent
        || user_agent.contains("Not mandatory")
    {
        tracing::info!("Terminal browser: {:?}", user_agent);
        let (response, branch_id) = raw_content(provider.clone(), &owner, &repository_name)
            .await
            .unwrap();
        update_statistic(provider, branch_id, user_agent).await;

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
                let (response, branch_id) =
                    raw_content(provider.clone(), &owner, &repository_name).await?;

                tracing::info!("Response is ready, branch_id = {}", branch_id);

                update_statistic(provider, branch_id, user_agent).await;
                Ok(response)
            } else {
                Response::builder()
                    .header("Content-Type", "text/plain")
                    .body(Body::from("/ws"))
                    .context(ResponseSnafu)
            }
        }
        None => static_page(),
    }
}

async fn update_statistic(
    provider: Arc<RwLock<GithubProvider>>,
    branch_id: DbId,
    user_agent: &str,
) {
    let provider_guard = provider.read().await;
    let connection = provider_guard.connection_pool.get().await.unwrap();
    let query = "INSERT INTO statistic VALUES(DEFAULT, $1, $2, NOW());";
    let r = connection
        .execute(query, &[&user_agent, &branch_id])
        .await
        .with_context(|_e| SqlQuerySnafu {
            query: query.to_string(),
        });

    match r {
        Ok(row_modified) => {
            tracing::info!("Insert to statistic. Row modified {row_modified}")
        }
        Err(error) => tracing::error!("Insert statistic error: {}", error.to_string()),
    }
}

async fn all_branches_lookup(
    Path((owner, repository_name)): Path<(String, String)>,
    Extension(provider): Extension<Arc<RwLock<GithubProvider>>>,
    _request: Request<Body>,
) -> Result<Response<Body>, Error> {
    let provider_guard = provider.read().await;

    let branches = provider_guard
        .remote_branches(&owner, &repository_name)
        .await
        .with_context(|_e| GithubProviderSnafu)?;

    tracing::debug!("{:?}", branches);

    let response = match serde_json::to_string(&branches) {
        Ok(body) => Response::builder()
            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
            .body(Body::from(body))
            .context(ResponseSnafu)?,
        Err(e) => Response::builder()
            .header(CONTENT_TYPE, TEXT_PLAIN.essence_str())
            .body(Body::from(e.to_string()))
            .context(ResponseSnafu)?,
    };
    Ok(response)
}

// recommendations from https://docs.github.com/en/rest/reference/branches
// const HEADER: (&str, &str) = ("Accept", "application/vnd.github.v3+json");

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Can't deserialize 'github API repo' JSON: {source}"))]
    DeserializeError { source: serde_json::Error },

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
        source: crate::providers::github::Error,
    },

    #[snafu(display("Error {source} at query {query}"))]
    SqlQuery {
        query: String,
        source: tokio_postgres::Error,
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
