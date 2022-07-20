use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::Response,
    Extension,
};

use crate::cloner::Cloner;

pub async fn handler_ws(
    Path((owner, repository_name, hostname)): Path<(String, String, String)>,
    ws: WebSocketUpgrade,
    Extension(cloner): Extension<Cloner>,
) -> Response {
    tracing::info!("AZOYAN {hostname} {owner} {repository_name}");
    ws.on_upgrade(move |socket| handle_socket(hostname, owner, repository_name, socket, cloner))
}

async fn handle_socket(
    hostname: String,
    owner: String,
    repository_name: String,
    mut socket: WebSocket,
    cloner: Cloner,
) {
    tracing::info!("Domain: {hostname}, Onwer: {owner}, Repo: {repository_name}");
    let repository_name = format!("https://{hostname}/{owner}/{repository_name}");
    let _repository = repository_name.clone();

    while let Some(msg) = socket.recv().await {
        if let Ok(_msg) = msg {
            let current_state = cloner.current_state(&repository_name).await;
            if current_state.is_empty() {
                socket.close().await;
                return;
                // socket.send(Message::Text("end".to_string())).await;
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
