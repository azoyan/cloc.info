use crate::{
    handlers::{self},
    logic::{
        git::Git,
        repository::{count_line_of_code, RepositoryProvider},
    },
    statistic::{largest, popular, recent},
    websocket::{handler_ws, handler_ws_with_branch},
};
use axum::{
    body::Body,
    error_handling::{HandleError, HandleErrorLayer},
    extract,
    handler::HandlerWithoutStateExt,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, get_service, post},
    serve::serve,
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Request, StatusCode};
use retainer::Cache;
use std::{future::IntoFuture, net::SocketAddr, sync::Arc, time::Duration};
use tempfile::tempdir_in;
use tokio::signal::{self, ctrl_c};
use tokio_postgres::NoTls;
use tokio_util::sync::CancellationToken;
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub async fn start_application(
    socket: SocketAddr,
    connection_pool: Pool<PostgresConnectionManager<NoTls>>,
) {
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
    let git_provider = Git::new(cache);

    let cancel = Arc::new(CancellationToken::new());
    let repository_provider = RepositoryProvider::new(
        connection_pool.clone(),
        git_provider.clone(),
        cancel.clone(),
    );

    let monitor =
        tokio::spawn(async move { cache_clone.monitor(4, 0.25, Duration::from_secs(1)).await });

    let websocket_service = Router::new()
        .route("/", get(handler_ws))
        .route("/tree/*branch", get(handler_ws_with_branch))
        .route("/-/tree/*branch", get(handler_ws_with_branch))
        .route("/src/*branch", get(handler_ws_with_branch))
        .route("/src/branch/*branch", get(handler_ws_with_branch))
        .with_state(repository_provider.clone());

    let statistic_router = Router::new()
        .route("/largest/:limit", get(largest))
        .route("/recent/:limit", get(recent))
        .route("/popular/:limit", get(popular))
        .with_state(connection_pool);

    let api_router = handlers::create_api_router(repository_provider.clone());
    let general_router = handlers::create_general_router(repository_provider.clone());

    let app = Router::new()
        .route_service("/", root_service)
        .route_service("/upload", upload_service)
        .route_service("/post", post(upload))
        .nest("/ws/:host/:owner/:repo", websocket_service)
        .nest("/api", statistic_router)
        .nest("/api/:host", api_router)
        .nest("/:host", general_router)
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

    let tcp_listener = tokio::net::TcpListener::bind(&socket).await.unwrap();
    let server = serve(tcp_listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(cancel, monitor))
        .into_future();

    let repository_service = repository_provider.run();
    let (_repo, handle) = tokio::join!(repository_service, server);

    match handle {
        Ok(_ok) => tracing::debug!("Exit"),
        Err(e) => tracing::error!("Error at stopping server: {e}"),
    }
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

async fn shutdown_signal(cancel: Arc<CancellationToken>, monitor: tokio::task::JoinHandle<()>) {
    let ctrl_c = async {
        ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    monitor.abort();

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    let hangup = async {
        signal::unix::signal(signal::unix::SignalKind::hangup())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!();
            tracing::debug!("SIGINT received, starting graceful shutdown");
        },
        _ = terminate => {
            println!();
            tracing::debug!("SIGTERM received, starting graceful shutdown");
        },
        _ = hangup => {
            println!();
            tracing::debug!("SIGHUP received, starting graceful shutdown");
        },
    }
    cancel.cancel();
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
    next: Next,
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
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

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
