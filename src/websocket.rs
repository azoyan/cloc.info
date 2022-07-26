use crate::cloner::Cloner;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::Response,
    Extension,
};

pub async fn handler_ws(
    ws: WebSocketUpgrade,
    Path(path): Path<String>,
    Extension(cloner): Extension<Cloner>,
    // request: Request<Body>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(path, socket, cloner))
}

async fn handle_socket(path: String, mut socket: WebSocket, cloner: Cloner) {
    tracing::info!("Path: {path}");
    let repository_name = format!("https://{}", &path[1..]); //  crop first slash

    while let Some(msg) = socket.recv().await {
        if let Ok(_msg) = msg {
            let current_state = cloner.current_state(&repository_name).await;
            if current_state.is_empty() {
                match socket.close().await {
                    Ok(()) => tracing::debug!("Connection 'ws://{repository_name}/ws' closed"),
                    Err(e) => tracing::error!("Error at closing connection: {}", e.to_string()),
                }
                return;
            }
            let msg = Message::Text(current_state);
            if socket.send(msg).await.is_err() {
                // client disconnected
                return;
            }
        } else {
            // client disconnected
            return;
        };
    }
}
