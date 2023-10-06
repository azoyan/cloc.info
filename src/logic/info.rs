use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, path::PathBuf};
use tokio_postgres::Row;

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub struct StorageInfo {
    pub size: usize,
    pub local_path: PathBuf,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
    Previous {
        date: DateTime<Utc>,
        commit: String,
        data: Vec<u8>,
    },
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
            Status::Previous { .. } => write!(f, "Previous"),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PopularBranch {
    branch_name: String,
    count: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PopularRepository {
    hostname: String,
    owner: String,
    repository_name: String,
    total_count: i64,
    branches: Vec<PopularBranch>,
}

impl PartialOrd for PopularRepository {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PopularRepository {
    fn cmp(&self, other: &Self) -> Ordering {
        other.total_count.cmp(&self.total_count)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PopularRepositories {
    pub repositories: Vec<PopularRepository>,
}

impl PopularRepositories {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(
        &mut self,
        hostname: String,
        owner: String,
        repository_name: String,
        branch: String,
        count: i64,
    ) {
        let branch = PopularBranch {
            branch_name: branch,
            count,
        };
        if let Some(repository) = self
            .repositories
            .iter_mut()
            .find(|r| r.repository_name == repository_name)
        {
            repository.total_count += count;

            repository.branches.push(branch);
        } else {
            let total_count = branch.count;
            let branches = vec![branch];
            let repository = PopularRepository {
                hostname,
                owner,
                repository_name,
                total_count,
                branches,
            };
            self.repositories.push(repository)
        }
        self.repositories.sort();
    }

    pub fn top(&self, limit: usize) -> Vec<PopularRepository> {
        self.repositories.iter().take(limit).cloned().collect()
    }
}

impl From<Vec<Row>> for PopularRepositories {
    fn from(rows: Vec<Row>) -> Self {
        let mut repositories = Self::default();

        for row in rows {
            let hostname: String = row.get("hostname");
            let owner: String = row.get("owner");
            let repository_name: String = row.get("repository_name");
            let branch: String = row.get("name");
            let count: i64 = row.get("count");
            repositories.push(hostname, owner, repository_name, branch, count);
        }
        repositories
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct LargestBranch {
    branch_name: String,
    size: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct LargestRepository {
    hostname: String,
    owner: String,
    repository_name: String,
    size: u64,
    branches: Vec<LargestBranch>,
}

impl PartialOrd for LargestRepository {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LargestRepository {
    fn cmp(&self, other: &Self) -> Ordering {
        other.size.cmp(&self.size)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct LargestRepositories {
    repositories: Vec<LargestRepository>,
}

impl LargestRepositories {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(
        &mut self,
        hostname: String,
        owner: String,
        repository_name: String,
        branch: String,
        size: u64,
    ) {
        let branch = LargestBranch {
            branch_name: branch,
            size,
        };
        if let Some(repository) = self
            .repositories
            .iter_mut()
            .find(|r| r.repository_name == repository_name)
        {
            repository.size = repository.size.max(size);
            repository.branches.push(branch);
        } else {
            let branches = vec![branch];
            let repository = LargestRepository {
                hostname,
                owner,
                repository_name,
                size,
                branches,
            };
            self.repositories.push(repository);
            self.repositories.sort();
        }
    }

    pub fn top(&self, limit: usize) -> Vec<LargestRepository> {
        self.repositories.iter().take(limit).cloned().collect()
    }
}

impl From<Vec<Row>> for LargestRepositories {
    fn from(rows: Vec<Row>) -> Self {
        let mut repositories = Self::default();

        for row in rows {
            let hostname: String = row.get("hostname");
            let owner: String = row.get("owner");
            let repository_name: String = row.get("repository_name");
            let branch: String = row.get("name");
            let size = row.get::<&str, i64>("size") as u64;
            repositories.push(hostname, owner, repository_name, branch, size);
        }
        repositories
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RecentBranch {
    branch_name: String,
    time: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RecentRepository {
    hostname: String,
    owner: String,

    repository_name: String,
    time: DateTime<Utc>,
    branches: Vec<RecentBranch>,
}

impl PartialOrd for RecentRepository {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecentRepository {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RecentRepositories {
    repositories: Vec<RecentRepository>,
}

impl RecentRepositories {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn push(
        &mut self,
        hostname: String,
        owner: String,
        repository_name: String,
        branch: String,
        time: DateTime<Utc>,
    ) {
        let branch = RecentBranch {
            branch_name: branch,
            time,
        };
        if let Some(repository) = self
            .repositories
            .iter_mut()
            .find(|r| r.repository_name == repository_name)
        {
            repository.time = repository.time.max(time);
            repository.branches.push(branch);
        } else {
            let branches = vec![branch];
            let repository = RecentRepository {
                hostname,
                owner,
                repository_name,
                time,
                branches,
            };
            self.repositories.push(repository);
            self.repositories.sort();
        }
    }

    pub fn top(&self, limit: usize) -> Vec<RecentRepository> {
        self.repositories.iter().take(limit).cloned().collect()
    }
}

impl From<Vec<Row>> for RecentRepositories {
    fn from(rows: Vec<Row>) -> Self {
        let mut repositories: RecentRepositories = RecentRepositories::new();

        for row in rows {
            let hostname: String = row.get("hostname");
            let owner: String = row.get("owner");
            let repository_name: String = row.get("repository_name");
            let branch: String = row.get("name");
            let time: DateTime<Utc> = row.get("time");
            repositories.push(hostname, owner, repository_name, branch, time);
        }
        repositories
    }
}
