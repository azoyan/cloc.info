use crate::{
    handlers::github::{
        get_branches, handle_github, handle_github_branch, handle_github_dummy, GithubProvider,
    },
    p404, script, welcome,
};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    error,
    http::{Method, StatusCode},
    middleware,
    rt::{self},
    web::{self, Data},
    App, HttpServer, Result,
};
use std::{io, net::SocketAddr};

pub struct Service {}

impl Service {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_on(self, socket_address: SocketAddr) -> Result<(), std::io::Error> {
        let data = Data::new(std::sync::RwLock::new(GithubProvider::new()));

        rt::System::new().block_on(
            HttpServer::new(move || {
                App::new()
                    .wrap(actix_cors::Cors::default().allow_any_origin())
                    .wrap(middleware::NormalizePath::trim())
                    .wrap(middleware::Compress::default())
                    .wrap(SessionMiddleware::new(
                        CookieSessionStore::default(),
                        Key::from(&[0; 64]),
                    ))
                    .wrap(middleware::Logger::default())
                    .service(
                        web::scope("/api")
                            .app_data(data.clone())
                            .service(get_branches),
                    )
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
                    .service(
                        actix_files::Files::new("/static", "./static")
                            .show_files_listing()
                            .use_last_modified(true),
                    )
                    .service(welcome)
                    // .service(web::resource("/git/{name}").route(web::get().to(with_param)))
                    // .service(web::resource("/async-body/{name}").route(web::get().to(response_body)))
                    .service(
                        web::resource("/index.js")
                            .to(|| async {
                                error::InternalError::new(
                                    io::Error::new(io::ErrorKind::Other, "test"),
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                )
                            })
                            .to(script),
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

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}
