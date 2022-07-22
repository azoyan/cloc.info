use bytes::BytesMut;
use std::{collections::HashMap, fmt::Display, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncReadExt, BufReader},
    process::Command,
    sync::RwLock,
};

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

    pub async fn clone_repository(&self, repository_name: &str, repository_path: &str) -> State {
        let mut command = Command::new("git");
        tracing::info!("clone {} to {}", repository_name, repository_path);

        command.args(&[
            "clone",
            "--progress",
            "--depth=1",
            repository_name,
            repository_path,
        ]);

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut reader = BufReader::new(stderr);
        let mut buffer = BytesMut::with_capacity(1000);
        // let mut buffer = String::new();

        // let mut buffer = [0u8; 1000];
        let mut stages = Stages::default();

        while reader.read_buf(&mut buffer).await.unwrap() != 0 {
            let slice = std::io::BufRead::split(&buffer[..], b'\r');
            for item in slice {
                let line = String::from_utf8(item.unwrap()).unwrap();

                if line.contains("Cloning") {
                    stages.cloning = String::from(&line);
                }
                if line.contains("remote: Enumerating") {
                    stages.enumerating = String::from(&line);
                }
                if line.contains("remote: Counting") {
                    stages.counting = String::from(&line);
                }
                if line.contains("remote: Compressing") {
                    stages.compressing = String::from(&line);
                }
                if line.contains("remote: Total") {
                    stages.total = String::from(&line);
                }
                if line.contains("Receiving") {
                    stages.receiving = String::from(&line);
                }
                if line.contains("Resolving") {
                    stages.resolving = String::from(&line);
                }
                if line.contains("Updating") {
                    stages.updating = String::from(&line);
                }
            }
            // let _res = std::io::Read::read_to_string(&mut slice, &mut line);
            // let mut line = String::from_utf8(slice).unwrap();
            // line.push('\\');
            // tracing::debug!("len:{}, line ={:?}", line, line.len());

            buffer.clear();
            // break;

            let stages_string = stages.to_string();

            tracing::debug!(
                "{repository_name} <-> ASCII:{} {}>>\n{}\n<<",
                stages_string.is_ascii(),
                stages_string.len(),
                &stages_string
            );
            self.clone_state
                .write()
                .await
                .insert(repository_name.to_string(), State::Buffered(stages_string));
            buffer.clear();
        }
        let _ = child.wait().await; // try
        State::Done
    }

    pub async fn current_state(&self, repository_name: &str) -> String {
        let clone_state = self.clone_state.read().await;
        let state = clone_state.get(repository_name);
        match state {
            Some(state) => match state {
                State::Buffered(stages) => {
                    // tracing::debug!(
                    //     "current state for repository = {}, \n>>\n{}\n<<",
                    //     repository_name,
                    //     &res
                    // );
                    stages.to_owned()
                }
                State::Done => String::from("DONE"),
            },
            None => format!("UNKNOWN REPOSITORY {}", repository_name),
        }
    }
}

impl Default for Cloner {
    fn default() -> Self {
        Self::new()
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
