use std::{collections::HashMap, fmt::Display, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::RwLock,
};

#[derive(Debug, Clone)]
pub enum State {
    Buffered(String),
    Done,
}

#[derive(Clone, Debug)]
pub struct Stages {
    cloning: String,
    enumerating: String,
    counting: String,
    compressing: String,
    total: String,
    receiving: String,
    resolving: String,
    updating: String,
}
impl Stages {
    pub fn new() -> Self {
        Self {
            cloning: String::new(),
            enumerating: String::new(),
            counting: String::new(),
            compressing: String::new(),
            total: String::new(),
            receiving: String::new(),
            resolving: String::new(),
            updating: String::new(),
        }
    }
}

impl Display for Stages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}{}{}",
            self.cloning,
            self.enumerating,
            self.counting,
            self.compressing,
            self.total,
            self.receiving,
            self.resolving,
            self.updating,
        )
    }
}

impl Default for Stages {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Cloner {
    clone_state: Arc<RwLock<HashMap<String, State>>>,
}

impl Cloner {
    pub fn new() -> Self {
        Self {
            clone_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn pull_repository(
        &self,
        url: &str,
        repository_path: &str,
        branch_name: &str,
    ) -> State {
        let mut command = Command::new("git");
        tracing::info!("pull {} to {}", url, repository_path);

        command.args(&[
            "-C",
            repository_path,
            "pull",
            "--ff-only",
            "--progress",
            "--allow-unrelated-histories",
            "origin",
            branch_name,
        ]);
        self.execute_pull(command, url).await
    }

    pub async fn execute_pull(&self, mut command: Command, url: &str) -> State {
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let stderr = child.stderr.take().unwrap();
        let mut reader = BufReader::new(stderr);

        let mut buffer = String::with_capacity(1000);

        let mut result = String::with_capacity(1000);
        while reader.read_line(&mut buffer).await.unwrap() != 0 {
            result.push_str(&buffer);

            tracing::debug!(
                "{url} <-> ASCII:{} {}>>\n{}\n<<",
                &result.is_ascii(),
                result.len(),
                &result
            );
            self.clone_state
                .write()
                .await
                .insert(url.to_string(), State::Buffered(result.clone()));
            buffer.clear();
        }
        let _c = child.wait().await; // try

        State::Done
    }

    pub async fn clone_repository(
        &self,
        url: &str,
        branch_name: &str,
        repository_path: &str,
    ) -> State {
        let mut command = Command::new("git");
        tracing::info!("clone --branch {branch_name} --depth 1 {} {}", url, repository_path);

        command.args(&[
            "clone",
            "--progress",
            "--depth=1",
            url,
            "--branch",
            branch_name,
            repository_path,
        ]);
        self.execute(command, url).await
    }

    pub async fn execute(&self, mut command: Command, url: &str) -> State {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut reader = BufReader::new(stderr);
        // let mut buffer = BytesMut::with_capacity(1000);
        let mut buffer = Vec::with_capacity(1000);
        // let mut buffer = String::new();

        // tracing::warn!("{:?}", command);

        // let mut buffer = [0u8; 1000];
        let mut stages = Stages::default();

        while reader.read_until(b'\r', &mut buffer).await.unwrap() != 0 {
            if let Ok(line) = String::from_utf8(buffer.clone()) {
                if line.contains("Cloning") {
                    stages.cloning = line;
                } else if line.contains("remote: Enumerating") {
                    stages.enumerating = line;
                } else if line.contains("remote: Counting") {
                    stages.counting = line;
                } else if line.contains("remote: Compressing") {
                    stages.compressing = line;
                } else if line.contains("remote: Total") {
                    stages.total = line;
                } else if line.contains("Receiving") {
                    stages.receiving = line;
                } else if line.contains("Resolving") {
                    stages.resolving = line;
                } else if line.contains("Updating") {
                    stages.updating = line;
                }
            }

            buffer.clear();

            let stages_string = stages.to_string();

            tracing::debug!(
                "ASCII:{} {}>>\n{}\n<<",
                stages_string.is_ascii(),
                stages_string.len(),
                &stages_string
            );
            self.clone_state
                .write()
                .await
                .insert(url.to_string(), State::Buffered(stages_string));
            buffer.clear();
        }
        // let _c = child.wait().await; // try

        State::Done
    }

    pub async fn set_done(&self, url: &str) {
        self.clone_state
            .write()
            .await
            .insert(url.to_string(), State::Done);
    }

    pub async fn current_state(&self, url: &str) -> Option<State> {
        let clone_state = self.clone_state.read().await;
        clone_state.get(url).cloned()
    }

    pub async fn clear_state_buffer(&self, url: &str) {
        let mut guard = self.clone_state.write().await;
        match guard.get_mut(url) {
            Some(state) => match state {
                State::Buffered(buffer) => buffer.clear(),
                State::Done => {}
            },
            None => {}
        }
    }
}

trait ContainsSlice<T>: PartialEq<[T]> {
    fn contains_slice(&'_ self, slice: &'_ [T]) -> bool;
}

impl<T, Item: PartialEq<T>> ContainsSlice<T> for [Item] {
    fn contains_slice(self: &'_ [Item], slice: &'_ [T]) -> bool {
        let len = slice.len();
        if len == 0 {
            return true;
        }
        self.windows(len).any(move |sub_slice| sub_slice == slice)
    }
}
