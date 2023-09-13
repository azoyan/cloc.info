use crate::{
    providers::repository_provider::RepositoryProvider,
    repository::info::{to_unique_name, to_url},
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};

pub async fn handler_ws(
    ws: WebSocketUpgrade,
    Path((host, owner, mut repository_name)): Path<(String, String, String)>,
    State(provider): State<RepositoryProvider>,
) -> impl IntoResponse {
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }

    let url = to_url(&host, &owner, &repository_name);
    let branch = provider.git_provider.default_branch(&url).await;

    match branch {
        Ok(branch) => ws.on_upgrade(move |socket| {
            handle_socket(
                host,
                owner,
                repository_name,
                branch,
                socket,
                State(provider),
            )
        }),
        Err(e) => panic!("{}", e.to_string()),
    }
}

pub async fn handler_ws_with_branch(
    ws: WebSocketUpgrade,
    Path((host, owner, mut repository_name, branch)): Path<(String, String, String, String)>,
    State(provider): State<RepositoryProvider>,
) -> Response {
    if host != "git.sr.ht" && !repository_name.ends_with(".git") {
        repository_name = format!("{repository_name}.git");
    }
    let branch = if host == "codeberg.org" {
        branch.trim_start_matches("/branch")
    } else {
        &branch
    };
    let branch = branch
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();

    ws.on_upgrade(move |socket| {
        handle_socket(
            host,
            owner,
            repository_name,
            branch.to_string(),
            socket,
            State(provider),
        )
    })
}

async fn handle_socket(
    host: String,
    owner: String,
    repository_name: String,
    branch: String,
    mut socket: WebSocket,
    provider: State<RepositoryProvider>,
) {
    let path = to_unique_name(&host, &owner, &repository_name, &branch);
    tracing::info!("Connect websocket {path}");

    while let Some(msg) = socket.recv().await {
        if let Ok(_msg) = msg {
            if let Some(state) = provider.cloner.current_state(&path).await {
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
                tracing::debug!("Not found repository {}", repository_name);
                match socket.close().await {
                    Ok(()) => {
                        tracing::debug!("Connection 'ws://{repository_name}/ws' closed")
                    }
                    Err(e) => {
                        tracing::error!("Error at closing connection: {}", e.to_string())
                    }
                }
                return;
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
