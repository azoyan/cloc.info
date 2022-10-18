use crate::repository::info::{BranchValue, Branches};
use retainer::Cache;
use snafu::{OptionExt, ResultExt, Snafu};
use std::{process::Command, string::FromUtf8Error, sync::Arc, time::Duration};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{desc}"))]
    BranchNotFound { desc: String },

    #[snafu(display("Error at getting {url}: {source}"))]
    IoError { url: String, source: std::io::Error },

    #[snafu(display("Can't deserialize git ls-remote output for {url} as utf8 string: {source}"))]
    Utf8 { url: String, source: FromUtf8Error },

    #[snafu(display("Error 'git ls-remote' output for {url}: {desc}"))]
    Line { url: String, desc: String },
}

#[derive(Clone)]
pub struct GitProvider {
    pub cache: Arc<Cache<String, Branches>>,
}

impl GitProvider {
    pub fn new(cache: Arc<Cache<String, Branches>>) -> Self {
        Self { cache }
    }

    pub async fn all_branches(&self, url: &String) -> Result<Branches, Error> {
        let branches = if let Some(branches) = self.cache.get(url).await {
            branches.clone()
        } else {
            tracing::info!("all_branches() Insert branches into git_provider cache for {url}");
            let branches = self::all_heads_branches(url)?;
            self.cache
                .insert(url.to_string(), branches.clone(), Duration::from_secs(60))
                .await;
            branches
        };

        Ok(branches)
    }

    pub async fn default_branch(&self, url: &String) -> Result<String, Error> {
        let branch = if let Some(branches) = self.cache.get(url).await {
            branches.default_branch.clone()
        } else {
            let branches = self.all_branches(url).await?;
            branches.default_branch
        };

        Ok(branch)
    }

    pub async fn last_commit(&self, url: &String, branch: &str) -> Result<String, Error> {
        if let Some(branches) = self.cache.get(url).await {
            for branch_value in &branches.branches {
                if branch_value.name == branch {
                    return Ok(branch_value.commit.clone());
                }
            }
            unreachable!("Branch {branch} not found in branches list for {url}");
        } else {
            let branches = self.all_branches(url).await?;
            for branch_value in &branches.branches {
                if branch_value.name == branch {
                    return Ok(branch_value.commit.clone());
                }
            }
            unreachable!("Branch {branch} not found in branches list for {url}");
        }
    }
}

pub fn all_heads_branches(url: &str) -> Result<Branches, Error> {
    let mut command = Command::new("git");

    let result = command
        .args(&["ls-remote", url])
        .output()
        .with_context(|_e| IoSnafu {
            url: url.to_string(),
        })?;

    if !result.status.success() {
        return Err(Error::BranchNotFound {
            desc: String::from_utf8(result.stderr).with_context(|_e| Utf8Snafu { url })?,
        });
    }

    let string = String::from_utf8(result.stdout).with_context(|_e| Utf8Snafu { url })?;
    let lines: Vec<&str> = string.lines().collect();

    let mut default_branch = String::new();
    let first_line = lines.first();
    let default_branch_commit = first_line.with_context(|| LineSnafu {
        url,
        desc: "No lines",
    })?;
    let default_branch_commit = default_branch_commit
        .split_whitespace()
        .next()
        .with_context(|| LineSnafu {
            url,
            desc: "Can't extract commit in splitted first line (HEAD)",
        })?;

    let filtered = lines.iter().filter(|line| line.contains("refs/heads/"));
    let mut branches = Vec::with_capacity(100);
    for line in filtered {
        let mut splitted = line.split_whitespace();
        let commit = splitted.next().with_context(|| LineSnafu {
            url,
            desc: "Can't extract commit",
        })?;
        let name = splitted
            .next()
            .with_context(|| LineSnafu {
                url,
                desc: "Can't extract branch ('refs/')",
            })?
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

    Ok(Branches {
        default_branch,
        branches,
    })
}
