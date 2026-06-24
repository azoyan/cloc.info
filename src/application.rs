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
use hyper::{Request, StatusCode};
use retainer::Cache;
use std::{future::IntoFuture, net::SocketAddr, path::Path, sync::Arc, time::Duration};
use tempfile::tempdir_in;
use tokio::{
    fs,
    signal::{self, ctrl_c},
};
use tokio_postgres::NoTls;
use tokio_util::sync::CancellationToken;
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

fn is_fingerprinted_asset(path: &str) -> bool {
    let Some(file_name) = path.rsplit('/').next() else {
        return false;
    };

    let Some((stem, _extension)) = file_name.rsplit_once('.') else {
        return false;
    };

    let Some((_, suffix)) = stem.rsplit_once('-') else {
        return false;
    };

    suffix.len() >= 6 && suffix.chars().all(|ch| ch.is_ascii_alphanumeric())
}

pub async fn start_application(
    socket: SocketAddr,
    connection_pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<(), String> {
    let root_service =
        get_service(ServeFile::new("dist/index.html")).handle_error(|error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        });

    let upload_service = HandleError::new(
        get_service(ServeFile::new("dist/upload.html")),
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
    let assets_service = get_service(ServeDir::new("dist/assets"))
        .handle_error(|error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        })
        // .layer(CompressionLayer::new().gzip(true).deflate(true).br(true))
        ;
    let app = Router::new()
        .route_service("/", root_service)
        .route_service("/upload", upload_service)
        .route_service("/post", post(upload))
        .nest("/ws/:host/:owner/:repo", websocket_service)
        .nest("/api", statistic_router)
        .nest("/api/:host", api_router)
        .nest("/:host", general_router)
        .nest_service("/assets", assets_service)
        .fallback_service(not_found.into_service())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_errors))
                .timeout(std::time::Duration::from_secs(600)),
        )
        .layer(CorsLayer::new().allow_credentials(true))
        .layer(axum::middleware::from_fn(set_static_cache_control))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let tcp_listener = tokio::net::TcpListener::bind(&socket)
        .await
        .map_err(|error| format!("Failed to bind HTTP listener at {socket}: {error}"))?;
    let server = serve(tcp_listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(cancel, monitor))
        .into_future();

    let repository_service = repository_provider.run();
    let (_repo, handle) = tokio::join!(repository_service, server);

    match handle {
        Ok(_ok) => {
            tracing::debug!("Exit");
            Ok(())
        }
        Err(error) => Err(format!("Error while running HTTP server: {error}")),
    }
}
async fn set_static_cache_control(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;

    if response
        .headers()
        .contains_key(hyper::header::CACHE_CONTROL)
    {
        return response;
    }

    let cache_control = if path.starts_with("/assets/") {
        if is_fingerprinted_asset(&path) {
            Some("public, max-age=31536000, immutable")
        } else {
            Some("no-cache,private,max-age=0")
        }
    } else if response
        .headers()
        .get(hyper::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"))
    {
        Some("no-cache,private,max-age=0")
    } else {
        None
    };

    if let Some(cache_control) = cache_control {
        response.headers_mut().insert(
            hyper::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static(cache_control),
        );
    }

    response
}

pub async fn not_found(_uri: axum::http::Uri) -> Response<Body> {
    match std::fs::read("dist/404.html") {
        Ok(buffer) => (
            StatusCode::NOT_FOUND,
            [
                ("Cache-Control", "no-cache,private,max-age=0"),
                ("Content-Type", "text/html; charset=utf-8"),
            ],
            buffer,
        )
            .into_response(),
        Err(error) => {
            tracing::error!("Can't load dist/404.html: {error}");
            (
                StatusCode::NOT_FOUND,
                [("Cache-Control", "no-cache,private,max-age=0")],
                "Not found",
            )
                .into_response()
        }
    }
}

async fn shutdown_signal(cancel: Arc<CancellationToken>, monitor: tokio::task::JoinHandle<()>) {
    let ctrl_c = async {
        if let Err(error) = ctrl_c().await {
            tracing::warn!("Failed to install Ctrl+C handler: {error}");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::warn!("Failed to install SIGTERM handler: {error}");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    #[cfg(unix)]
    let hangup = async {
        match signal::unix::signal(signal::unix::SignalKind::hangup()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::warn!("Failed to install SIGHUP handler: {error}");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let hangup = std::future::pending::<()>();

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
    monitor.abort();
    if let Err(error) = monitor.await {
        if !error.is_cancelled() {
            tracing::warn!("Cache monitor task stopped unexpectedly: {error}");
        }
    }
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

async fn upload(mut multipart: extract::Multipart) -> Result<Response<Body>, (StatusCode, String)> {
    fs::create_dir_all("cloc_repo")
        .await
        .map_err(internal_server_error)?;

    let tempdir = tempdir_in("cloc_repo").map_err(internal_server_error)?;
    let path = tempdir.path().to_path_buf();
    let path_str = path
        .to_str()
        .ok_or_else(|| internal_server_error("temporary path is not valid UTF-8"))?;

    let mut index = 0usize;
    while let Some(field) = multipart.next_field().await.map_err(bad_request)? {
        let source_name = field.file_name().or(field.name()).unwrap_or("upload");
        let relative_path = sanitize_upload_path(source_name);
        let data = field.bytes().await.map_err(bad_request)?;
        let file_path = unique_upload_path(&path, &relative_path, index);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(internal_server_error)?;
        }

        fs::write(&file_path, &data)
            .await
            .map_err(internal_server_error)?;

        tracing::debug!("write file '{}'", file_path.display());
        index += 1;
    }

    let scc_output = count_line_of_code(path_str, "")
        .await
        .map_err(|e| internal_server_error(e.to_string()))?;

    Response::builder()
        .header("Content-Type", "text/plain")
        .body(Body::from(scc_output))
        .map_err(internal_server_error)
}

fn sanitize_upload_path(name: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::new();

    for component in Path::new(name).components() {
        if let std::path::Component::Normal(value) = component {
            let Some(value) = value.to_str() else {
                continue;
            };

            let sanitized = value
                .chars()
                .map(|ch| if ch.is_control() { '_' } else { ch })
                .collect::<String>();

            if !sanitized.is_empty() {
                path.push(sanitized);
            }
        }
    }

    if path.as_os_str().is_empty() {
        path.push("upload");
    }

    path
}

fn unique_upload_path(root: &Path, relative_path: &Path, index: usize) -> std::path::PathBuf {
    let candidate = root.join(relative_path);

    if !candidate.exists() {
        return candidate;
    }

    let parent = candidate
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());
    let stem = candidate
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("upload");
    let extension = candidate.extension().and_then(|value| value.to_str());
    let file_name = match extension {
        Some(extension) if !extension.is_empty() => format!("{index:04}_{stem}.{extension}"),
        _ => format!("{index:04}_{stem}"),
    };

    parent.join(file_name)
}

fn bad_request(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, error.to_string())
}

fn internal_server_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
