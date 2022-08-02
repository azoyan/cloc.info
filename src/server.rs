use crate::{github, providers::github_provider::GithubProvider, MB};
use axum::{
    error_handling::HandleErrorLayer,
    handler::Handler,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get_service,
    Extension, Router,
};
use axum_extra::routing::SpaRouter;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bytes::Bytes;
use hyper::{Body, Method, Request, StatusCode, Uri};
use retainer::Cache;
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
    let github_provider = GithubProvider::new(4 * crate::GB, connection_pool, cache);

    let _monitor =
        tokio::spawn(async move { cache_clone.monitor(4, 0.25, Duration::from_secs(3)).await });

    let websocket_service = Router::new().route(
        "/:id/*path",
        axum::routing::get(crate::websocket::handler_ws)
            .layer(Extension(github_provider.cloner.clone())),
    );
    let gh_provider = Arc::new(RwLock::new(github_provider));
    let app = Router::new()
        .route("/", root_service)
        .nest("/ws", websocket_service)
        .nest(
            "/api/github.com",
            github::create_api_router(gh_provider.clone()),
        )
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
        // .layer(axum::middleware::from_fn(print_request_response))
        .layer(TraceLayer::new_for_http());

    let handle = axum_server::Handle::new();

    tokio::spawn(graceful_shutdown(handle.clone()));

    axum_server::bind(socket)
        .handle(handle)
        .serve(app.into_make_service())
}

async fn handle_error(method: Method, uri: Uri, err: std::io::Error) -> String {
    format!("{} {} failed with {}", method, uri, err)
}

pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route {}", uri),
    )
}

pub async fn not_found(uri: axum::http::Uri) -> Response<Body> {
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
