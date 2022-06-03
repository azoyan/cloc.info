#![feature(map_first_last)]

pub mod handlers;
pub mod repository;
pub mod service;

use crate::service::Service;
use actix_files as fs;
use actix_session::Session;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpRequest, Result};

use std::net::{IpAddr, SocketAddr};

/// favicon handler
#[get("/{file}")]
async fn favicon(path: web::Path<String>) -> Result<fs::NamedFile> {
    let path = format!("static/{}", path.into_inner());
    Ok(fs::NamedFile::open(path)?)
}

/// simple index handler
#[get("/")]
async fn welcome(session: Session, req: HttpRequest) -> Result<fs::NamedFile> {
    println!("{:?}", req);

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        counter = count + 1;
    }

    // set counter to session
    session.insert("counter", counter)?;

    Ok(fs::NamedFile::open("static/index.html")?.set_status_code(StatusCode::OK))
}

/// 404 handler
async fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

fn main() -> Result<(), std::io::Error> {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    struct Opt {
        /// IP address of service
        host_ip: std::net::Ipv4Addr,
        /// Port of service
        host_port: u16,
    }

    let opt = Opt::from_args();
    let ip = IpAddr::V4(opt.host_ip);
    let port = opt.host_port;

    let socket = SocketAddr::new(ip, port);

    // env::set_var("RUST_LOG", "actix_web=debug,actix_server=debug");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let service = Service::new();
    service.run_on(socket)
}
