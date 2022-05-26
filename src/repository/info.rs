use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub struct StorageInfo {
    pub size: usize,
    pub local_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryInfo {
    pub hostname: String,
    pub owner: String,
    pub repository_name: String,
    pub branch: String,
    pub last_commit: String,
    pub local_path: PathBuf,
    pub size: u64,
    pub scc_output: Vec<u8>,
}

impl RepositoryInfo {
    pub fn to_url(&self) -> String {
        let Self {
            hostname,
            owner,
            repository_name,
            branch,
            ..
        } = &self;
        to_url(hostname, owner, repository_name, branch)
    }

    pub fn to_filename(&self) -> String {
        let Self {
            hostname,
            owner,
            repository_name,
            branch,
            ..
        } = &self;
        to_filename(hostname, owner, repository_name, branch)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub commit: Commit,
    pub protected: bool,
    pub protection: Protection,
    pub protection_url: String,
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
    format!("{hostname}_{owner}_{repository_name}_branch_{branch}")
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub struct CocomoInfo {
    pub cost_develop: String,
    pub schedule_effort: String,
    pub people_required: String,
}
