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
    Path((id, path)): Path<(String, String)>,
    Extension(cloner): Extension<Cloner>,
    // request: Request<Body>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(id, path, socket, cloner))
}

async fn handle_socket(id: String, path: String, mut socket: WebSocket, cloner: Cloner) {
    tracing::info!("Connect websocket {id}/{path}");
    let repository_name = format!("https://{}", &path[1..]); //  crop first slash
    cloner.clear_state_buffer(&repository_name).await;
    while let Some(msg) = socket.recv().await {
        if let Ok(_msg) = msg {
            if let Some(state) = cloner.current_state(&repository_name).await {
                match state {
                    crate::cloner::State::Buffered(state) => {
                        // tracing::debug!("Websocket {id}/{path} send: {}", state);
                        let msg = Message::Text(state);
                        if socket.send(msg).await.is_err() {
                            // client disconnected
                            return;
                        }
                    }
                    crate::cloner::State::Done => {
                        tracing::debug!("Repository {} downloading done", repository_name);
                        match socket.send(Message::Text("Done".to_string())).await {
                            Ok(()) => {
                                tracing::debug!("Connection {id}/{repository_name} closed")
                            }
                            Err(e) => {
                                tracing::warn!("{} {id}/{repository_name}", e.to_string())
                            }
                        }
                        return;
                    }
                }
            } else {
                // tracing::debug!("Not found repository {}", repository_name);
                // match socket.close().await {
                //     Ok(()) => {
                //         tracing::debug!("Connection 'ws://{repository_name}/ws' closed")
                //     }
                //     Err(e) => {
                //         tracing::error!("Error at closing connection: {}", e.to_string())
                //     }
                // }
                // return;
            }
        } else {
            // client disconnected
            match socket.close().await {
                Ok(()) => tracing::debug!(
                    "Can't receive message {}. Connection {id}/{repository_name}' closed",
                    repository_name
                ),
                Err(e) => tracing::warn!("{} {id}/{repository_name}", e.to_string()),
            }
            return;
        };
    }
}
