use snafu::Snafu;
use std::{
    path::Path,
    process::{Command, Stdio},
    str::from_utf8,
};

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

use String as CommitHash;

pub(crate) fn last_commit_local(path: &str) -> Result<CommitHash, Error> {
    let mut last_commit_command = std::process::Command::new("git");
    last_commit_command.args(["-C", path, "rev-parse", "HEAD"]);

    match last_commit_command.output() {
        Ok(output) if output.status.success() => match from_utf8(&output.stdout) {
            Ok(hash) => Ok(hash.trim().to_string()),
            Err(e) => Err(Error::LastCommitError {
                repository: path.to_string(),
                error: e.to_string(),
            }),
        },
        Ok(output) => {
            let status_code = match output.status.code() {
                Some(code) => code.to_string(),
                None => String::from("None"),
            };

            Err(Error::LastCommitError {
                repository: path.to_string(),
                error: format!("git status code: {status_code}"),
            })
        }
        Err(e) => Err(Error::LastCommitError {
            repository: path.to_string(),
            error: e.to_string(),
        }),
    }
}

pub async fn count_line_of_code(path: &str, _format: &str) -> Result<Vec<u8>, Error> {
    let mut scc_command = tokio::process::Command::new("scc");
    tracing::debug!("Counting line of code in path: {path}");
    scc_command.args(["--ci", path]);
    let out = match scc_command.output().await {
        Ok(output) if !output.status.success() => {
            let error = String::from_utf8(output.stderr)
                .unwrap_or_else(|e| format!("Error at convert git output to utf8: {e}"));
            tracing::error!("scc error: {}", error);
            return Err(Error::SccError { error });
        }
        Ok(output) => output.stdout,
        Err(e) => {
            tracing::error!("scc error: {e}");
            return Err(Error::SccError {
                error: e.to_string(),
            });
        }
    };
    // tracing::info!("{:?}", out);
    Ok(out)
}

pub fn dir_size<P>(path: P) -> Result<u64, Error>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_str().unwrap();
    let mut command = Command::new("du");
    command.args(["-sb", path]);

    command.stdout(Stdio::piped());

    match command.output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(out) => match out.split_whitespace().next() {
                Some(str) => match str.parse::<u64>() {
                    Err(e) => Err(Error::SizeError {
                        error: e.to_string(),
                    }),
                    Ok(size) => Ok(size),
                },
                None => Err(Error::SizeError {
                    error: format!("Wrong `du` stdout: {out}"),
                }),
            },
            Err(e) => Err(Error::SizeError {
                error: e.to_string(),
            }),
        },
        Err(e) => Err(Error::SizeError {
            error: e.to_string(),
        }),
    }
}
