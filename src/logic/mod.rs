pub mod cloner;
pub mod git;
pub mod info;
pub mod repository;

use snafu::Snafu;
use std::string::FromUtf8Error;

type Id = i64;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Can't deserialize bytes: {bytes} from request {url} {source}"))]
    DeserializeError {
        bytes: String,
        url: String,
        source: serde_json::Error,
    },

    #[snafu(display("Repository not found by API request {url}"))]
    NotFound { url: String },

    #[snafu(display("Error at API request {url} message: {message}"))]
    RemoteError { url: String, message: String },

    #[snafu(display("Can't extract default branch for repository {repo}"))]
    ExtractDefaultBranchError { repo: String },

    #[snafu(display("Template page not found"))]
    TemplatePage,

    #[snafu(display("Can't create request '{url}': {source}"))]
    CreateRequest {
        url: String,
        source: hyper::http::Error,
    },

    #[snafu(display("Can't send request '{url}': {source}"))]
    SendRequest { url: String, source: hyper::Error },

    #[snafu(display("Can't get response body '{url}': {source}"))]
    GetResponseBody { url: String, source: hyper::Error },

    #[snafu(display("Branch '{wrong_branch}' is note exist"))]
    WrongBranch { wrong_branch: String },

    #[snafu(display("Error {source} at query {query}"))]
    Query {
        query: String,
        source: tokio_postgres::Error,
    },

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
    Size { error: String },

    #[snafu(display("Can't create temporary directory: {source}"))]
    CreateTempDirError { source: std::io::Error },

    #[snafu(display("Repository downloading already in progress"))]
    InProgress { url: String },

    #[snafu(display("{desc}"))]
    BranchNotFound { desc: String },

    #[snafu(display("Error at getting {url}: {source}"))]
    Io { url: String, source: std::io::Error },

    #[snafu(display("Error at remove dir {unique_name}: {source}"))]
    RemoveDir {
        unique_name: String,
        source: std::io::Error,
    },

    #[snafu(display("Can't deserialize git ls-remote output for {url} as utf8 string: {source}"))]
    Utf8 { url: String, source: FromUtf8Error },

    #[snafu(display("Error 'git ls-remote' output for {url}: {desc}"))]
    Line { url: String, desc: String },

    #[snafu(display("Rejected insertion to disk cache for {path}"))]
    Rejected { path: String },
}
