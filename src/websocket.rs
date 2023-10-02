use crate::logic::{
    info::{to_unique_name, to_url, Status},
    repository::RepositoryProvider,
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
    let unique_name = to_unique_name(&host, &owner, &repository_name, &branch);
    tracing::info!("Connect websocket {}", unique_name);

    while let Some(msg) = socket.recv().await {
        if let Ok(_msg) = msg {
            let state = provider.current_status(&unique_name);
            let msg = match serde_json::to_string(&state) {
                Ok(json) => Message::Text(json),
                Err(e) => Message::Text(e.to_string()),
            };

            if socket.send(msg).await.is_err() || matches!(state, Status::Done(_)) {
                return; // client disconnected
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
