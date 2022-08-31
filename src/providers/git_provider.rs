use crate::repository::info::{BranchValue, Branches};
use retainer::Cache;
use std::{process::Command, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct GitProvider {
    pub cache: Arc<Cache<String, Branches>>,
}

impl GitProvider {
    pub fn new(cache: Arc<Cache<String, Branches>>) -> Self {
        Self { cache }
    }

    pub async fn all_branches(&self, url: &String) -> Branches {
        if let Some(branches) = self.cache.get(url).await {
            branches.clone()
        } else {
            tracing::info!("all_branches() Insert branches into git_provider cache for {url}");
            let branches = self::all_heads_branches(url).await;
            self.cache
                .insert(url.to_string(), branches.clone(), Duration::from_secs(60))
                .await;
            branches
        }
    }

    pub async fn default_branch(&self, url: &String) -> String {
        if let Some(branches) = self.cache.get(url).await {
            branches.default_branch.clone()
        } else {
            let branches = self.all_branches(url).await;
            branches.default_branch
        }
    }

    pub async fn last_commit(&self, url: &String, branch: &str) -> String {
        if let Some(branches) = self.cache.get(url).await {
            for branch_value in &branches.branches {
                if branch_value.name == branch {
                    return branch_value.commit.clone();
                }
            }
            unreachable!();
        } else {
            let branches = self.all_branches(url).await;
            for branch_value in &branches.branches {
                if branch_value.name == branch {
                    return branch_value.commit.clone();
                }
            }
            unreachable!();
        }
    }
}

pub async fn all_heads_branches(url: &str) -> Branches {
    let mut command = Command::new("git");

    let result = command.args(&["ls-remote", url]).output().unwrap();
    let string = String::from_utf8(result.stdout).unwrap();
    let lines: Vec<&str> = string.lines().collect();

    let mut default_branch = String::new();
    let first_line = lines.get(0);
    let default_branch_commit = first_line.unwrap();
    let default_branch_commit = default_branch_commit.split_whitespace().next().unwrap();

    let filtered = lines.iter().filter(|line| line.contains("refs/heads/"));
    let mut branches = Vec::with_capacity(100);
    for line in filtered {
        let mut splitted = line.split_whitespace();
        let commit = splitted.next().unwrap();
        let name = splitted
            .next()
            .unwrap()
            .trim_start_matches("refs/heads/")
            .to_string();

        if commit == default_branch_commit {
            default_branch = name.clone();
        }

        branches.push(BranchValue {
            name,
            commit: commit.to_string(),
        })
    }

    Branches {
        default_branch,
        branches,
    }
}

pub async fn last_commit(url: &str, branch: &str) -> String {
    let mut command = Command::new("git");
    let result = command.args(&["ls-remote", url]).output().unwrap();
    let string = String::from_utf8(result.stdout).unwrap();
    let lines = string.lines();

    let filtered = lines.filter(|line| line.contains("refs/heads/"));
    for line in filtered {
        let mut splitted = line.split_whitespace();
        let commit = splitted.next().unwrap();
        let name = splitted
            .next()
            .unwrap()
            .trim_start_matches("refs/heads/")
            .to_string();

        if name == branch {
            return commit.to_string();
        }
    }
    unreachable!()
}
