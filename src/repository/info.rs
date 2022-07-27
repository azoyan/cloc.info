use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchValue {
    pub name: String,
    pub commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Branches {
    pub default_branch: Option<String>,
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

pub fn to_url(hostname: &str, owner: &str, repository_name: &str, branch: &str) -> String {
    format!("https://{hostname}/{owner}/{repository_name}/{branch}")
}

pub fn to_filename(hostname: &str, owner: &str, repository_name: &str, branch: &str) -> String {
    let branch = branch.replace('/', "_");
    format!("{hostname}_{owner}_{repository_name}_branch_{branch}")
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
