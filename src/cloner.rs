use std::{collections::HashMap, fmt::Display, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::RwLock,
};

use crate::repository::info::{to_unique_name, to_url};

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

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
struct Repo {
    host: String,
    owner: String,
    name: String,
    branch: String,
}

struct Task(Vec<String>);
impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for s in &self.0 {
            result.push_str(s);
            result.push(' ');
        }
        write!(f, "{}", result.trim_end())
    }
}
impl AsRef<Vec<String>> for Task {
    fn as_ref(&self) -> &Vec<String> {
        self.0.as_ref()
    }
}

#[derive(Clone, Default)]
pub struct Cloner {
    clone_state: Arc<RwLock<HashMap<String, State>>>,
}

impl Cloner {
    pub fn new() -> Self {
        Default::default()
    }

    async fn execute_new(
        &self,
        Repo {
            host,
            owner,
            name,
            branch,
        }: Repo,
        task: Task,
    ) -> State {
        let mut command = Command::new("git");
        command.args(task.as_ref());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let unique_name = to_unique_name(&host, &owner, &name, &branch);
        tracing::debug!("git {task} unique_name: {unique_name}");

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

            // tracing::debug!(
            //     "ASCII:{} {}>>\n{}\n<<",
            //     stages_string.is_ascii(),
            //     stages_string.len(),
            //     &stages_string
            // );

            self.clone_state
                .write()
                .await
                .insert(unique_name.clone(), State::Buffered(stages_string));
            buffer.clear();
        }
        // let _c = child.wait().await; // try

        State::Done
    }

    pub async fn pull_repository(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
        repository_path: &str,
        branch_name: &str,
    ) -> State {
        let task = Task(vec![
            "-C".to_string(),
            repository_path.to_string(),
            "pull".to_string(),
            "--ff-only".to_string(),
            "--progress".to_string(),
            "--allow-unrelated-histories".to_string(),
            "origin".to_string(),
            branch_name.to_string(),
        ]);
        let repo = Repo {
            host: host.to_string(),
            owner: owner.to_string(),
            name: repository_name.to_string(),
            branch: branch_name.to_string(),
        };
        self.execute_new(repo, task).await
    }

    pub async fn execute_pull(&self, mut command: Command, unique_name: &str) -> State {
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let stderr = child.stderr.take().unwrap();
        let mut reader = BufReader::new(stderr);

        let mut buffer = String::with_capacity(1000);

        let mut result = String::with_capacity(1000);
        while reader.read_line(&mut buffer).await.unwrap() != 0 {
            result.push_str(&buffer);

            tracing::debug!(
                "{unique_name} <-> ASCII:{} {}>>\n{}\n<<",
                &result.is_ascii(),
                result.len(),
                &result
            );
            self.clone_state
                .write()
                .await
                .insert(unique_name.to_string(), State::Buffered(result.clone()));
            buffer.clear();
        }
        let _c = child.wait().await; // try

        State::Done
    }

    pub async fn clone_repository(
        &self,
        host: &str,
        owner: &str,
        repository_name: &str,
        branch_name: &str,
        repository_path: &str,
    ) -> State {
        let url = to_url(host, owner, repository_name);
        // tracing::info!(
        //     "clone --branch {branch_name} --depth 1 {} {}",
        //     url,
        //     repository_path
        // );

        let task = Task(vec![
            "clone".to_string(),
            "--progress".to_string(),
            "--depth=1".to_string(),
            url,
            "--branch".to_string(),
            branch_name.to_string(),
            repository_path.to_string(),
        ]);
        let repo = Repo {
            host: host.to_string(),
            owner: owner.to_string(),
            name: repository_name.to_string(),
            branch: branch_name.to_string(),
        };
        self.execute_new(repo, task).await
    }

    pub async fn execute(
        &self,
        mut command: Command,
        host: &str,
        owner: &str,
        repository_name: &str,
        branch: &str,
    ) -> State {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let unique_name = to_unique_name(host, owner, repository_name, branch);

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
                .insert(unique_name.clone(), State::Buffered(stages_string));
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
        if let Some(state) = guard.get_mut(url) {
            match state {
                State::Buffered(buffer) => buffer.clear(),
                State::Done => {}
            }
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
