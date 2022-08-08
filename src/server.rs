use crate::{github, providers::github_provider::GithubProvider};
use axum::{
    error_handling::HandleErrorLayer,
    extract::Path,
    handler::Handler,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Extension, Router,
};
use axum_extra::routing::SpaRouter;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use hyper::{header::CONTENT_TYPE, Body, Method, Request, StatusCode, Uri};
use mime_guess::mime::APPLICATION_JSON;
use retainer::Cache;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_postgres::NoTls;
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeFile, trace::TraceLayer,
};

pub fn create_server(
    socket: SocketAddr,
    connection_pool: Pool<PostgresConnectionManager<NoTls>>,
) -> impl futures::Future<Output = Result<(), std::io::Error>> {
    let root_service = get_service(ServeFile::new("static/index.html")).handle_error(
        |error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        },
    );
    let spa = SpaRouter::new("/static", "static").handle_error(handle_error);

    let cache = Arc::new(Cache::new());
    let cache_clone = cache.clone();
    let github_provider = GithubProvider::new(4 * crate::GB, connection_pool.clone(), cache);

    let _monitor =
        tokio::spawn(async move { cache_clone.monitor(4, 0.25, Duration::from_secs(3)).await });

    let websocket_service = Router::new().route(
        "/:id/*path",
        axum::routing::get(crate::websocket::handler_ws)
            .layer(Extension(github_provider.cloner.clone())),
    );

    let statistic_router = Router::new()
        .route("/largest/:limit", get(largest))
        .route("/recent/:limit", get(recent))
        .route("/popular/:limit", get(popular))
        .layer(Extension(connection_pool));
    let gh_provider = Arc::new(RwLock::new(github_provider));
    let app = Router::new()
        .route("/", root_service)
        .nest("/ws", websocket_service)
        .nest(
            "/api/github.com",
            github::create_api_router(gh_provider.clone()),
        )
        .nest("/api", statistic_router)
        .nest("/github.com", github::create_router(gh_provider))
        .merge(spa)
        .fallback(not_found.into_service())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_errors))
                .timeout(std::time::Duration::from_secs(600)),
        )
        .layer(CorsLayer::new().allow_credentials(true))
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(axum::middleware::from_fn(print_request_response))
        .layer(TraceLayer::new_for_http());

    let handle = axum_server::Handle::new();

    tokio::spawn(graceful_shutdown(handle.clone()));

    axum_server::bind(socket)
        .handle(handle)
        .serve(app.into_make_service())
}

async fn largest(
    Path(limit): Path<i64>,
    Extension(connection_pool): Extension<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();
    let result = pool
        .query(
            "select * from all_view order by size desc limit $1",
            &[&limit],
        )
        .await;

    match result {
        Ok(rows) => {
            let mut res = Vec::with_capacity(rows.len());

            for row in rows {
                let hostname: String = row.get("hostname");
                let owner: String = row.get("owner");
                let repository_name: String = row.get("repository_name");
                let branch: String = row.get("name");
                let size: i64 = row.get("size");
                let value = json!({
                    "hostname": hostname,
                    "owner": owner,
                    "repository_name": repository_name,
                    "branch_name": branch,
                    "size": size,
                });
                res.push(value);
            }
            let res = json!(res);
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(res.to_string()))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}

async fn recent(
    Path(limit): Path<i64>,
    Extension(connection_pool): Extension<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();

    let result = pool
        .query(
            "select * from all_view order by time desc limit $1",
            &[&limit],
        )
        .await;

    match result {
        Ok(rows) => {
            let mut res = Vec::with_capacity(rows.len());

            for row in rows {
                let hostname: String = row.get("hostname");
                let owner: String = row.get("owner");
                let repository_name: String = row.get("repository_name");
                let branch: String = row.get("name");
                let time: DateTime<Utc> = row.get("time");
                let value = json!({
                    "hostname": hostname,
                    "owner": owner,
                    "repository_name": repository_name,
                    "branch_name": branch,
                    "time": time.to_rfc3339(),
                });
                res.push(value);
            }
            let res = json!(res);
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(res.to_string()))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}

async fn popular(
    Path(limit): Path<i64>,
    Extension(connection_pool): Extension<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();

    let result = pool
        .query("select * from popular_repositories limit $1", &[&limit])
        .await;

    match result {
        Ok(rows) => {
            let mut res = Vec::with_capacity(rows.len());

            for row in rows {
                let hostname: String = row.get("hostname");
                let owner: String = row.get("owner");
                let repository_name: String = row.get("repository_name");
                let branch: String = row.get("name");
                let count: i64 = row.get("count");
                let value = json!({
                    "hostname": hostname,
                    "owner": owner,
                    "repository_name": repository_name,
                    "branch_name": branch,
                    "count": count,
                });
                res.push(value);
            }
            let res = json!(res);
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(res.to_string()))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}

async fn handle_error(method: Method, uri: Uri, err: std::io::Error) -> String {
    format!("{} {} failed with {}", method, uri, err)
}

pub async fn not_found(_uri: axum::http::Uri) -> Response<Body> {
    let file = std::fs::File::open("static/404.html").unwrap();
    let mut reader = std::io::BufReader::new(file);

    let mut buffer = vec![];

    std::io::Read::read_to_end(&mut reader, &mut buffer).unwrap();
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Cache-Control", "no-cache,private,max-age=0") // TODO, maybe rewrite AddHeader
        .body(Body::from(buffer))
        .unwrap()
}

async fn graceful_shutdown(handle: axum_server::Handle) {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    handle.shutdown();
    tracing::info!("signal shutdown");
}

async fn handle_errors(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}
