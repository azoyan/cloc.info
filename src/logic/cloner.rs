use super::Error;
use crate::logic::info::{to_url, Status, Task};
use dashmap::DashMap;
use std::{fmt::Display, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[derive(Clone, Debug)]
pub struct Stages {
    command: String,
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
            command: String::new(),
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
            "{}{}{}{}{}{}{}{}{}",
            self.command,
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

struct Args(Vec<String>);
impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for s in &self.0 {
            result.push_str(s);
            result.push(' ');
        }
        write!(f, "{}", result.trim_end())
    }
}
impl AsRef<Vec<String>> for Args {
    fn as_ref(&self) -> &Vec<String> {
        self.0.as_ref()
    }
}

#[derive(Clone, Default)]
pub struct Cloner {
    statuses: Arc<DashMap<String, Status>>,
}

impl Cloner {
    pub fn new(statuses: Arc<DashMap<String, Status>>) -> Self {
        Self { statuses }
    }

    async fn execute_new(
        &self,
        args: Args,
        unique_name: &str,
        path: &str,
    ) -> Result<Status, Error> {
        let mut command = Command::new("git");
        command.args(args.as_ref());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        tracing::debug!("{command:?} to {path}");

        let mut child = command.spawn().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut reader = BufReader::new(stderr);
        let mut buffer = Vec::with_capacity(1000);
        let mut stages = Stages {
            command: format!("git {}\n", args),
            ..Default::default()
        };

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

            buffer.clear();
            self.statuses
                .insert(unique_name.to_string(), Status::InProgress(stages_string));
        }

        match child.wait().await {
            Ok(exit) => {
                tracing::debug!("git clone exit code: {}", exit);
                if exit.success() {
                    Ok(Status::Cloned)
                } else {
                    Err(Error::CloneError {
                        repository: path.into(),
                        error: exit.to_string(),
                    })
                }
            }
            Err(e) => {
                tracing::error!("git clone exit error: {}", e);
                Err(Error::CloneError {
                    repository: path.into(),
                    error: e.to_string(),
                })
            }
        }
    }

    // pub async fn pull_repository(&self, task: &Task, destination: &str) -> Result<Status, Error> {
    //     let args = Args(vec![
    //         "-C".to_string(),
    //         destination.to_string(),
    //         "pull".to_string(),
    //         "--ff-only".to_string(),
    //         "--progress".to_string(),
    //         "--depth=1".to_string(),
    //         "--allow-unrelated-histories".to_string(),
    //         "origin".to_string(),
    //         task.branch.clone(),
    //     ]);
    //     self.execute_new(args, destination).await
    // }

    pub async fn clone_repository(
        &self,
        task: &Task,
        unique_name: &str,
        path: &str,
    ) -> Result<Status, Error> {
        let url = to_url(&task.host, &task.owner, &task.repository_name);
        let args = Args(vec![
            "clone".to_string(),
            "--progress".to_string(),
            "--depth=1".to_string(),
            "--branch".to_string(),
            task.branch.clone(),
            url,
            path.to_string(),
        ]);
        self.execute_new(args, unique_name, path).await
    }

    pub async fn set_done(&self, unique_name: &str) {
        self.statuses
            .insert(unique_name.to_string(), Status::Cloned);
    }

    pub async fn current_state(&self, name: &str) -> Option<Status> {
        self.statuses.get(name).map(|kv| kv.value().clone())
    }

    pub async fn clear_state_buffer(&self, unique_name: &str) {
        if let Some(mut state) = self.statuses.get_mut(unique_name) {
            if let Status::InProgress(buffer) = state.value_mut() {
                buffer.clear();
            };
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
