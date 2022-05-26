use super::info::{to_url, CocomoInfo, RepositoryInfo};
use snafu::Snafu;
use std::{path::PathBuf, process::Command, str::from_utf8};
use tempdir::TempDir;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error at creating temporary directory for clone: {error}"))]
    TempDirError { error: String },

    #[snafu(display("Error at cloning (git clone) repository {repository}: {error}"))]
    CloneError { repository: String, error: String },

    #[snafu(display(
        "Error at extracting last commit (git rev-parse HEAD) of repository {repository}: {error}"
    ))]
    LastCommitError { repository: String, error: String },

    #[snafu(display("Error at counting line of code (scc): {error}"))]
    SccError { error: String },
}

pub async fn cloc_branch(
    hostname: &str,
    owner: &str,
    repository_name: &str,
    branch: &str,
) -> Result<RepositoryInfo, Error> {
    let dir_name = format!("{hostname}_{owner}_{repository_name}_{branch}");
    let (_temp_dir, path) = create_temp_dir(&dir_name)?;
    let last_commit = clone(&to_url(hostname, owner, repository_name, ""), &path).await?;
    let scc_output = count_line_of_code(&path, "").await.unwrap();

    let size = fs_extra::dir::get_size(&path).unwrap();

    let repository_info = RepositoryInfo {
        hostname: hostname.to_string(),
        owner: owner.to_string(),
        repository_name: repository_name.to_string(),
        branch: branch.to_string(),
        last_commit,
        local_path: PathBuf::from(path),
        size,
        scc_output,
    };

    Ok(repository_info)
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
                error: format!("Can't transform tmp_dir to path"),
            })
        }
    };

    Ok((tmp_dir, path))
}

pub async fn clone(url: &str, path: &str) -> Result<String, Error> {
    let mut git_clone_command = Command::new("git");
    let repository = url.to_string();

    git_clone_command.args(&["clone", "--progress", "--depth=1", url, path]);

    match git_clone_command.output() {
        Ok(output) if !output.status.success() => {
            return Err(Error::CloneError {
                repository,
                error: String::from_utf8(output.stderr)
                    .unwrap_or(String::from("Error at convert git output to utf8")),
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

    let mut last_commit_command = Command::new("git");
    last_commit_command.args(["-C", path, "rev-parse", "HEAD"]);

    match last_commit_command.output() {
        Ok(output) if output.status.success() => match from_utf8(&output.stdout) {
            Ok(hash) => Ok(hash.trim().to_string()),
            Err(e) => Err(Error::LastCommitError {
                repository,
                error: e.to_string(),
            }),
        },
        Ok(output) => {
            let status_code = match output.status.code() {
                Some(code) => code.to_string(),
                None => String::from("None"),
            };

            Err(Error::LastCommitError {
                repository,
                error: format!("git status code: {status_code}"),
            })
        }
        Err(e) => Err(Error::LastCommitError {
            repository,
            error: e.to_string(),
        }),
    }
}

async fn count_line_of_code(path: &str, format: &str) -> Result<Vec<u8>, Error> {
    let mut scc_command = Command::new("scc");
    scc_command.args(["--ci", "--wide", "--format-multi", format, &path]);
    let out = match scc_command.output() {
        Ok(output) if !output.status.success() => {
            return Err(Error::SccError {
                error: String::from_utf8(output.stderr)
                    .unwrap_or(String::from("Error at convert git output to utf8")),
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

fn parse_output(lines: &[&str]) -> Result<(), Error> {
    let json = serde_json::Value::from(lines[0]);

    println!("len = {}", lines.len());
    let last_index = lines.len() - 1;
    let header = lines[1].split_whitespace().collect::<Vec<&str>>().join(",");

    println!("{header}");
    let languages = &lines[3..last_index - 9];
    for language in languages.iter() {
        println!("{language}");
    }

    let total = lines[last_index - 7]
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(",");
    let people_required = lines[last_index - 3];
    let schedule_effort = lines[last_index - 4];
    let cost_develop = lines[last_index - 5];

    println!("{total}");
    println!("{cost_develop}");
    println!("{schedule_effort}");
    println!("{people_required}");
    // let (left, right) = processed.split_once(',').unwrap();

    // let mut splited = left.split_whitespace();
    // let next = splited.next();
    // let bytes = splited.next().unwrap().parse::<usize>().unwrap();

    let cocomo_info = CocomoInfo {
        cost_develop: cost_develop.to_string(),
        schedule_effort: schedule_effort.to_string(),
        people_required: people_required.to_string(),
    };
    Ok(())
}

fn parse_output2(lines: Vec<&str>) -> Result<(serde_json::Value, CocomoInfo, usize), Error> {
    let json = serde_json::Value::from(lines[0]);

    let last_index = lines.len() - 1;
    let processed = lines[last_index - 4];
    let people_required = lines[last_index - 6];
    let schedule_effort = lines[last_index - 7];
    let cost_develop = lines[last_index - 8];

    let (left, right) = processed.split_once(',').unwrap();

    let mut splited = left.split_whitespace();
    let next = splited.next();
    let bytes = splited.next().unwrap().parse::<usize>().unwrap();

    let cocomo_info = CocomoInfo {
        cost_develop: cost_develop.to_string(),
        schedule_effort: schedule_effort.to_string(),
        people_required: people_required.to_string(),
    };

    Ok((json, cocomo_info, bytes))
}

// async fn json_response(url: &str) -> Result<HttpResponse> {
//     let (_tmp_dir, tmp_dir_path) = create_temp_dir(url)?;
//     clone(url, &tmp_dir_path).await?;
//     let bytes = count_line_of_code(tmp_dir_path, "json").await?;
//     Ok(HttpResponse::Ok()
//         .content_type("application/json")
//         .body(bytes))
// }

// async fn html_respnose(url: &str) -> Result<HttpResponse> {
//     let (_tmp_dir, tmp_dir_path) = create_temp_dir(url)?;
//     clone(url, &tmp_dir_path).await?;
//     let bytes = count_line_of_code(tmp_dir_path, "html").await?;
//     Ok(HttpResponse::Ok().content_type("text/html").body(bytes))
// }

// async fn ascii_respnose(url: &str) -> Result<HttpResponse> {
//     let (_tmp_dir, tmp_dir_path) = create_temp_dir(url)?;
//     clone(url, &tmp_dir_path).await?;
//     let bytes = count_line_of_code(tmp_dir_path, "html").await?;
//     Ok(HttpResponse::Ok().content_type("plain/text").body(bytes))
// }

#[cfg(test)]
mod tests {
    use super::parse_output;

    #[test]
    fn parse() {
        let s = include_str!("../../tmp.txt");

        let lines: Vec<&str> = s.lines().collect();
        parse_output(&lines);
    }
}
