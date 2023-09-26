use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, sync::Arc};
use tempfile::TempDir;

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub struct StorageInfo {
    pub size: usize,
    pub local_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LocalTempDir {
    pub temp_dir: Arc<TempDir>,
    pub path: String,
}

impl LocalTempDir {
    pub fn new(temp_dir: TempDir) -> Self {
        let path = temp_dir.path().to_str().unwrap().to_owned();
        Self {
            temp_dir: Arc::new(temp_dir),
            path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepositoryInfo {
    pub hostname: String,
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BranchValue {
    pub name: String,
    pub commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Branches {
    pub default_branch: String,
    pub branches: Vec<BranchValue>,
}

impl RepositoryInfo {
    pub fn new(hostname: &str, owner: &str, name: &str) -> Self {
        Self {
            hostname: hostname.to_owned(),
            owner: owner.to_owned(),
            name: name.to_owned(),
        }
    }
}

impl PartialEq for LocalTempDir {
    fn eq(&self, other: &Self) -> bool {
        self.temp_dir.path().eq(other.temp_dir.path()) && self.path.eq(&other.path)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub commit: Commit,
    pub protected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commit {
    pub sha: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Protection {
    pub required_status_checks: RequiredStatusChecks,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequiredStatusChecks {
    pub enforcement_level: String,
    pub contexts: Vec<String>,
}

pub fn to_url(hostname: &str, owner: &str, repository_name: &str) -> String {
    format!("https://{hostname}/{owner}/{repository_name}")
}

pub fn to_unique_name(host: &str, owner: &str, repository_name: &str, branch: &str) -> String {
    format!("{host}/{owner}/{repository_name}/{branch}")
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub struct CocomoInfo {
    pub cost_develop: String,
    pub schedule_effort: String,
    pub people_required: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllBranchesInfo {
    pub default_branch: String,
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub host: String,
    pub owner: String,
    pub repository_name: String,
    pub branch: String,
    pub default_branch: String,
    pub user_agent: String,
}

impl Task {
    pub fn to_unique_name(&self) -> String {
        to_unique_name(&self.host, &self.owner, &self.repository_name, &self.branch)
    }

    pub fn to_path(&self) -> String {
        format!("{}/{}/{}", self.host, self.owner, self.repository_name)
    }

    pub fn to_url(&self) -> String {
        to_url(&self.host, &self.owner, &self.repository_name)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    InProgress(String),
    Cloned,
    Done(Vec<u8>),
    Ready,
    Error(String),
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Done(_) => write!(f, "Done"),
            Status::InProgress(text) => write!(f, "{}", text),
            Status::Cloned => write!(f, "Cloned"),
            Status::Ready => write!(f, "Ready"),
            Status::Error(e) => write!(f, "Error: {e}"),
        }
    }
}
