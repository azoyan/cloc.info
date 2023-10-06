#![forbid(unsafe_code)]

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use clap::Parser;
use cloc::application::start_application;
use std::net::{IpAddr, SocketAddr};
use time::{format_description, UtcOffset};
use tokio_postgres::NoTls;
use tracing_subscriber::{fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    let timer = format_description::parse(
        "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]:[subsecond digits:3]",
    )
    .expect("Cataplum");
    let time_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let timer = OffsetTime::new(time_offset, timer);

    let layer = tracing_subscriber::fmt::layer().compact().with_timer(timer);

    //Set the RUST_LOG, if it hasn't been explicitly defined
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or("cloc=trace,tower_http=trace".into()),
        ))
        .with(layer)
        .init();

    #[derive(Debug, Parser)]
    #[command(author, version, about)]
    struct Opt {
        /// IP address of service
        ip_address: std::net::Ipv4Addr,
        /// Port of service
        port: u16,
    }

    let opt = Opt::parse();
    let ip = IpAddr::V4(opt.ip_address);
    let port = opt.port;

    let socket = SocketAddr::new(ip, port);

    let r = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("main_thread")
        .build()
        .unwrap();

    let path = std::env::current_exe().unwrap();

    tracing::info!("Path: {path:?}. Starting cloc server {ip}:{port}");
    r.block_on(start_all(socket));
}

async fn start_all(socket: SocketAddr) {
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost user=postgres dbname=clocdb",
        NoTls,
    )
    .unwrap();
    let _config = tokio_postgres::Config::new()
        .dbname("clocdb")
        .host("localhost")
        .user("postgres")
        .to_owned();
    // let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();

    start_application(socket, pool.clone()).await;
}
