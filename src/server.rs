use crate::{
    application,
    providers::{git_provider::GitProvider, repository_provider::RepositoryProvider},
    repository::utils::count_line_of_code,
    statistic::{largest, popular, recent},
    websocket::{handler_ws, handler_ws_with_branch},
};
use axum::{
    error_handling::{HandleError, HandleErrorLayer},
    extract,
    handler::HandlerWithoutStateExt,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, get_service, post},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bytes::Bytes;
use hyper::{Body, Request, StatusCode};
use retainer::Cache;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tempfile::tempdir_in;
use tokio::sync::RwLock;
use tokio_postgres::NoTls;
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub fn create_server(
    socket: SocketAddr,
    connection_pool: Pool<PostgresConnectionManager<NoTls>>,
) -> impl futures::Future<Output = Result<(), std::io::Error>> {
    let root_service =
        get_service(ServeFile::new("static/index.html")).handle_error(|error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        });

    let upload_service = HandleError::new(
        get_service(ServeFile::new("static/upload.html")),
        |error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        },
    );

    let cache = Arc::new(Cache::new());
    let cache_clone = cache.clone();
    let git_provider = GitProvider::new(cache);
    let github_provider = RepositoryProvider::new(
        16 * crate::GB,
        connection_pool.clone(),
        git_provider.clone(),
    );

    let _monitor =
        tokio::spawn(async move { cache_clone.monitor(4, 0.25, Duration::from_secs(3)).await });

    let state = (github_provider.cloner.clone(), git_provider.clone());

    let websocket_service = Router::new()
        .route("/", get(handler_ws))
        .route("/tree/*branch", get(handler_ws_with_branch))
        .route("/-/tree/*branch", get(handler_ws_with_branch))
        .route("/src/*branch", get(handler_ws_with_branch))
        .with_state(state);

    let statistic_router = Router::new()
        .route("/largest/:limit", get(largest))
        .route("/recent/:limit", get(recent))
        .route("/popular/:limit", get(popular))
        .with_state(connection_pool);

    let gh_provider = Arc::new(RwLock::new(github_provider));

    let api_router = application::create_api_router(gh_provider.clone());
    let router = application::create_router(gh_provider);

    let app = Router::new()
        .route_service("/", root_service)
        .route_service("/upload", upload_service)
        .route_service("/post", post(upload))
        .nest("/ws/:host/:owner/:repo", websocket_service)
        .nest("/api", statistic_router)
        .nest("/api/:host", api_router)
        .nest("/:host", router)
        .nest_service("/static", ServeDir::new("static"))
        .fallback_service(not_found.into_service())
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

    Ok(bytes)
}

async fn upload(mut multipart: extract::Multipart) -> Response<Body> {
    let tempdir = tempdir_in("cloc_repo").unwrap();
    let path = tempdir.path().to_str().unwrap();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        std::fs::write(&format!("{path}/{name}"), &data).unwrap();

        tracing::debug!("write file '{name}' ")
    }
    let scc_output = count_line_of_code(path, "").await.unwrap();
    Response::builder()
        .header("Content-Type", "text/plain")
        .body(Body::from(scc_output))
        .unwrap()
}
