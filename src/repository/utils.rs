use super::info::{to_url, LocalTempDir};
use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use snafu::Snafu;
use std::{process::Command, str::from_utf8};
use tempdir::TempDir;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error at creating temporary directory for clone: {error}"))]
    TempDirError { error: String },

    #[snafu(display("Error at cloning (git clone) repository {repository}: {error}"))]
    CloneError { repository: String, error: String },

    #[snafu(display("Error at pulling (git pull) repository {url}: {error}"))]
    PullError { url: String, error: String },

    #[snafu(display(
        "Error at extracting last commit (git rev-parse HEAD) of repository {repository}: {error}"
    ))]
    LastCommitError { repository: String, error: String },

    #[snafu(display("Error at counting line of code (scc): {error}"))]
    SccError { error: String },

    #[snafu(display("Error at getting size of directory: {error}"))]
    SizeError { error: String },
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match self {
            Error::CloneError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SccError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::TempDirError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::LastCommitError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SizeError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::PullError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
use String as CommitHash;

pub async fn clone_branch(
    hostname: &str,
    owner: &str,
    repository_name: &str,
    branch: &str,
) -> Result<(CommitHash, LocalTempDir, u64), Error> {
    let dir_name = format!("{hostname}_{owner}_{repository_name}_{branch}");
    let (temp_dir, path) = create_temp_dir(&dir_name)?;
    let last_commit = clone(&to_url(hostname, owner, repository_name, ""), &path).await?;

    let size = fs_extra::dir::get_size(&path).map_err(|e| Error::SizeError {
        error: e.to_string(),
    })?;

    Ok((last_commit, LocalTempDir::new(temp_dir), size))
}

fn create_temp_dir(dir_name: &str) -> Result<(TempDir, String), Error> {
    let tmp_dir = match TempDir::new_in(".", dir_name) {
        Ok(dir) => dir,
        Err(e) => {
            return Err(Error::TempDirError {
                error: e.to_string(),
            })
        }
    };

    let path = match tmp_dir.path().to_str() {
        Some(path) => path.to_string(),
        None => {
            return Err(Error::TempDirError {
                error: "Can't transform tmp_dir to path".to_string(),
            })
        }
    };

    Ok((tmp_dir, path))
}

pub async fn pull(url: &str, path: &str, branch: &str) -> Result<(CommitHash, u64), Error> {
    let mut git_pull_command = Command::new("git");
    git_pull_command.args(&["pull", "-C", path, "origin", branch]);

    match git_pull_command.output() {
        Ok(output) if !output.status.success() => {
            return Err(Error::PullError {
                url: url.to_string(),
                error: String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| String::from("Error at convert git output to utf8")),
            })
        }
        Err(e) => {
            return Err(Error::PullError {
                url: url.to_string(),
                error: e.to_string(),
            })
        }
        Ok(_output) => {}
    };
    let last_commit = last_commit_local(url.to_string(), path)?;

    let size = fs_extra::dir::get_size(&path).map_err(|e| Error::SizeError {
        error: e.to_string(),
    })?;

    Ok((last_commit, size))
}

pub async fn clone(url: &str, path: &str) -> Result<CommitHash, Error> {
    let mut git_clone_command = Command::new("git");
    let repository = url.to_string();

    git_clone_command.args(&["clone", "--progress", "--depth=1", url, path]);

    match git_clone_command.output() {
        Ok(output) if !output.status.success() => {
            return Err(Error::CloneError {
                repository,
                error: String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| String::from("Error at converting git output to utf8")),
            })
        }
        Err(e) => {
            return Err(Error::CloneError {
                repository,
                error: e.to_string(),
            })
        }
        Ok(_output) => {}
    };

    last_commit_local(repository, path)
}

fn last_commit_local(url: String, path: &str) -> Result<CommitHash, Error> {
    let mut last_commit_command = Command::new("git");
    last_commit_command.args(["-C", path, "rev-parse", "HEAD"]);

    match last_commit_command.output() {
        Ok(output) if output.status.success() => match from_utf8(&output.stdout) {
            Ok(hash) => Ok(hash.trim().to_string()),
            Err(e) => Err(Error::LastCommitError {
                repository: url,
                error: e.to_string(),
            }),
        },
        Ok(output) => {
            let status_code = match output.status.code() {
                Some(code) => code.to_string(),
                None => String::from("None"),
            };

            Err(Error::LastCommitError {
                repository: url,
                error: format!("git status code: {status_code}"),
            })
        }
        Err(e) => Err(Error::LastCommitError {
            repository: url,
            error: e.to_string(),
        }),
    }
}

pub async fn count_line_of_code(path: &str, format: &str) -> Result<Vec<u8>, Error> {
    let mut scc_command = Command::new("scc");
    scc_command.args(["--ci", "--wide", "--format-multi", format, path]);
    let out = match scc_command.output() {
        Ok(output) if !output.status.success() => {
            return Err(Error::SccError {
                error: String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| String::from("Error at convert git output to utf8")),
            })
        }
        Ok(output) => output.stdout,
        Err(e) => {
            return Err(Error::SccError {
                error: e.to_string(),
            })
        }
    };

    Ok(out)
}
