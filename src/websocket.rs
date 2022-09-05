use crate::{
    cloner::Cloner,
    providers::git_provider::GitProvider,
    repository::info::{to_unique_name, to_url},
};
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
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    Extension(cloner): Extension<Cloner>,
    Extension(provider): Extension<GitProvider>,
    // request: Request<Body>,
) -> Response {
    if !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }

    let url = to_url(&host, &owner, &repository_name);
    let branch = provider.default_branch(&url).await;

    match branch {
        Ok(branch) => {
            let path = to_unique_name(&host, &owner, &repository_name, &branch);
            ws.on_upgrade(move |socket| handle_socket(path, socket, cloner))
        }
        Err(e) => panic!("{}", e.to_string()),
    }
}

pub async fn handler_ws_with_branch(
    ws: WebSocketUpgrade,
    Path((host, owner, mut repository_name, branch)): Path<(String, String, String, String)>,
    Extension(cloner): Extension<Cloner>,
    Extension(_provider): Extension<GitProvider>,
    // request: Request<Body>,
) -> Response {
    if !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let branch = branch.trim_start_matches("/").trim_end_matches("/");
    let path = to_unique_name(&host, &owner, &repository_name, &branch);
    ws.on_upgrade(move |socket| handle_socket(path, socket, cloner))
}

async fn handle_socket(path: String, mut socket: WebSocket, cloner: Cloner) {
    tracing::info!("Connect websocket {path}");
    let repository_name = path;
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
                                tracing::debug!("Connection {repository_name} closed")
                            }
                            Err(e) => {
                                tracing::warn!("{} {repository_name}", e.to_string())
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
                    "Can't receive message {}. Connection '{repository_name}' closed",
                    repository_name
                ),
                Err(e) => tracing::warn!("{} {repository_name}", e.to_string()),
            }
            return;
        };
    }
}
