use super::{
    info::{BranchValue, Branches},
    {Error, LineSnafu, Utf8Snafu},
};
use retainer::Cache;
use snafu::{OptionExt, ResultExt};
use std::{sync::Arc, time::Duration};
use tokio::process::Command;

#[derive(Clone)]
pub struct Git {
    pub cache: Arc<Cache<String, Branches>>,
}

impl Git {
    pub fn new(cache: Arc<Cache<String, Branches>>) -> Self {
        Self { cache }
    }

    pub async fn all_branches(&self, url: &str) -> Result<Branches, Error> {
        let branches = if let Some(branches) = self.cache.get(&url.to_string()).await {
            tracing::info!("Get branches from cache");
            branches.clone()
        } else {
            let branches = self::all_heads_branches(url).await?;
            self.cache
                .insert(url.to_string(), branches.clone(), Duration::from_secs(60))
                .await;
            tracing::info!("all_branches() Inserted branches into git_provider cache for {url}");
            branches
        };

        Ok(branches)
    }

    pub async fn default_branch(&self, url: &str) -> Result<String, Error> {
        let branch = if let Some(branches) = self.cache.get(&url.to_string()).await {
            tracing::info!("Get branch {} from cache", branches.default_branch);
            branches.default_branch.clone()
        } else {
            let branches = self.all_branches(url).await?;
            branches.default_branch
        };

        Ok(branch)
    }

    pub async fn last_commit(&self, url: &str, branch: &str) -> Result<String, Error> {
        let branches = if let Some(branches) = self.cache.get(&url.to_string()).await {
            branches.clone()
        } else {
            self.all_branches(url).await?
        };

        branches
            .branches
            .into_iter()
            .find(|branch_value| branch_value.name == branch)
            .map(|branch_value| branch_value.commit)
            .ok_or_else(|| Error::BranchNotFound {
                desc: format!("Branch '{branch}' not found in branches list for {url}"),
            })
    }
}

pub async fn all_heads_branches(url: &str) -> Result<Branches, Error> {
    let mut command = Command::new("git");

    let result = command
        .args(["ls-remote", url])
        .output()
        .await
        .map_err(|e| Error::Io {
            url: url.into(),
            source: e,
        })?;

    if !result.status.success() {
        return Err(Error::BranchNotFound {
            desc: String::from_utf8(result.stderr).context(Utf8Snafu { url })?,
        });
    }

    let string = String::from_utf8(result.stdout).context(Utf8Snafu { url })?;
    let lines: Vec<&str> = string.lines().collect();

    let mut default_branch = String::new();
    let first_line = lines.first();
    let default_branch_commit = first_line.context(LineSnafu {
        url,
        desc: "No lines",
    })?;
    let default_branch_commit =
        default_branch_commit
            .split_whitespace()
            .next()
            .context(LineSnafu {
                url,
                desc: "Can't extract commit in splitted first line (HEAD)",
            })?;

    let filtered = lines.iter().filter(|line| line.contains("refs/heads/"));
    let mut branches = Vec::with_capacity(100);
    for line in filtered {
        let mut splitted = line.split_whitespace();
        let commit = splitted.next().context(LineSnafu {
            url,
            desc: "Can't extract commit",
        })?;
        let name = splitted
            .next()
            .context(LineSnafu {
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

#[cfg(test)]
mod tests {
    use super::Git;
    use crate::logic::{
        info::{BranchValue, Branches},
        Error,
    };
    use retainer::Cache;
    use std::{sync::Arc, time::Duration};

    #[tokio::test]
    async fn missing_branch_returns_error() {
        let url = "https://example.com/org/repo.git";
        let cache = Arc::new(Cache::new());
        cache
            .insert(
                url.to_string(),
                Branches {
                    default_branch: "main".to_string(),
                    branches: vec![BranchValue {
                        name: "main".to_string(),
                        commit: "abc123".to_string(),
                    }],
                },
                Duration::from_secs(60),
            )
            .await;

        let git = Git::new(cache);
        let error = git.last_commit(url, "missing").await.unwrap_err();

        assert!(matches!(error, Error::BranchNotFound { .. }));
    }
}
