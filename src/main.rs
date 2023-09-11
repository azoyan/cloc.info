#![feature(byte_slice_trim_ascii)]

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use cloc::server::create_server;
use std::net::{IpAddr, SocketAddr};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    //Set the RUST_LOG, if it hasn't been explicitly defined
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cloc=trace,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    let r = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("main_thread")
        .build()
        .unwrap();

    tracing::info!("Starting...");
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
    let server = tokio::task::spawn(create_server(socket, pool));
    let handle = tokio::join!(server);
    drop(handle);
}
