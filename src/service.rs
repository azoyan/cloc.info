use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    error, guard,
    http::{header::ContentType, Method, StatusCode},
    middleware,
    rt::{self},
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Result,
};
use derive_more::{Display, Error};
use std::{io, net::SocketAddr, process::Command, sync::RwLock};
use tempdir::TempDir;

use crate::{
    handlers::github::{handle_github, handle_github_branch, handle_github_dummy, GithubProvider},
    p404,
    repository::cache::RepositoryCache,
    response_body, welcome,
};

pub struct Service {
    repository_provider: RepositoryCache,
}

impl Service {
    pub fn new(cache_size: usize) -> Self {
        Self {
            repository_provider: RepositoryCache::new(cache_size),
        }
    }

    pub fn run_on(self, socket_address: SocketAddr) -> Result<(), std::io::Error> {
        let data = Data::new(std::sync::RwLock::new(GithubProvider::new()));

        rt::System::new().block_on(
            HttpServer::new(move || {
                App::new()
                    .wrap(SessionMiddleware::new(
                        CookieSessionStore::default(),
                        Key::from(&[0; 64]),
                    ))
                    .wrap(middleware::Logger::default())
                    .service(
                        web::scope("/github.com")
                            // .guard(guard::Header("content-type", "text/plain"))
                            // .app_data(Data::clone(&data))
                            .app_data(data.clone())
                            .service(handle_github)
                            .service(handle_github_branch)
                            .service(handle_github_dummy),
                    )
                    // .service(web::scope("/gitlab.com").service(handle_gitlab))
                    // .serive(web::scope("/api").service(handle_api))
                    .service(welcome)
                    // .service(web::resource("/git/{name}").route(web::get().to(with_param)))
                    // .service(web::resource("/async-body/{name}").route(web::get().to(response_body)))
                    .service(
                        web::resource("/test").to(|req: HttpRequest| match *req.method() {
                            Method::GET => HttpResponse::Ok(),
                            Method::POST => HttpResponse::MethodNotAllowed(),
                            _ => HttpResponse::NotFound(),
                        }),
                    )
                    .service(web::resource("/error").to(|| async {
                        error::InternalError::new(
                            io::Error::new(io::ErrorKind::Other, "test"),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    }))
                    // .service(fs::Files::new("/static", "static").show_files_listing())
                    // .service(web::resource("/").route(web::get().to(response_body)))
                    .default_service(web::route().method(Method::GET).to(p404))
            })
            .bind(socket_address)?
            .run(),
        )
    }
}

#[derive(Debug, Display, Error)]
enum UserError {
    #[display(fmt = "Error at creating temporary directory for clone: {error}")]
    TempDirError { error: String },
    #[display(fmt = "Error at cloning (git clone) repository {repository}: {error}")]
    CloneError { repository: String, error: String },
    #[display(fmt = "Error at counting line of code (scc): {error}")]
    SccError { error: String },
}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::CloneError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::SccError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::TempDirError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn create_temp_dir(_repository: &str) -> Result<(TempDir, String), UserError> {
    let tmp_dir = match TempDir::new_in(".", "tmp") {
        Ok(dir) => dir,
        Err(e) => {
            return Err(UserError::TempDirError {
                error: e.to_string(),
            })
        }
    };

    let path = match tmp_dir.path().to_str() {
        Some(path) => path.to_string(),
        None => {
            return Err(UserError::TempDirError {
                error: format!("Can't transform tmp_dir to path"),
            })
        }
    };

    Ok((tmp_dir, path))
}

async fn clone(url: &str, path: &str) -> Result<(), UserError> {
    let mut git_clone_command = Command::new("git");
    let repository = url.to_string();

    git_clone_command.args(&["clone", "--progress", "--depth=1", url, path]);

    match git_clone_command.output() {
        Ok(output) if !output.status.success() => Err(UserError::CloneError {
            repository,
            error: String::from_utf8(output.stderr)
                .unwrap_or(String::from("Error at convert git output to utf8")),
        }),
        Err(e) => Err(UserError::CloneError {
            repository,
            error: e.to_string(),
        }),
        _ => Ok(()),
    }
}

async fn count_line_of_code(path: String, format: &str) -> Result<Vec<u8>, UserError> {
    let mut scc_command = Command::new("scc");
    scc_command.args(["--ci", "-f", format, &path]);
    match scc_command.output() {
        Ok(output) if !output.status.success() => Err(UserError::SccError {
            error: String::from_utf8(output.stderr)
                .unwrap_or(String::from("Error at convert git output to utf8")),
        }),
        Ok(output) => Ok(output.stdout),
        Err(e) => Err(UserError::SccError {
            error: e.to_string(),
        }),
    }
}

async fn json_response(url: &str) -> Result<HttpResponse> {
    let (_tmp_dir, tmp_dir_path) = create_temp_dir(url).await?;
    clone(url, &tmp_dir_path).await?;
    let bytes = count_line_of_code(tmp_dir_path, "json").await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(bytes))
}

async fn html_respnose(url: &str) -> Result<HttpResponse> {
    let (_tmp_dir, tmp_dir_path) = create_temp_dir(url).await?;
    clone(url, &tmp_dir_path).await?;
    let bytes = count_line_of_code(tmp_dir_path, "html").await?;
    Ok(HttpResponse::Ok().content_type("text/html").body(bytes))
}

async fn ascii_respnose(url: &str) -> Result<HttpResponse> {
    let (_tmp_dir, tmp_dir_path) = create_temp_dir(url).await?;
    clone(url, &tmp_dir_path).await?;
    let bytes = count_line_of_code(tmp_dir_path, "html").await?;
    Ok(HttpResponse::Ok().content_type("plain/text").body(bytes))
}

// #[get("/gitlab.com/{tail:.*}")]
// async fn handle_gitlab(request: HttpRequest, path: web::Path<String>) -> HttpResponse {}
