#![forbid(unsafe_code)]

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use clap::Parser;
use cloc::application::start_application;
use const_format::formatcp;
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

    const VERSION: &str = formatcp!(
        "\n\n\
        Version:             {}\n\
         Description:         {}\n\
         Build Timestamp:     {}\n\
         Commit SHA:          {}\n\
         Commit Message:      \"{}\"\n\
         rustc Version:       {}\n\
         cargo Target Triple: {}\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("VERGEN_BUILD_TIMESTAMP"),
        env!("VERGEN_GIT_DESCRIBE"),
        env!("VERGEN_GIT_COMMIT_MESSAGE"),
        env!("VERGEN_RUSTC_SEMVER"),
        env!("VERGEN_CARGO_TARGET_TRIPLE")
    );

    #[derive(Debug, Parser)]
    #[command(version = VERSION)]
    #[command(about)]
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
    // Read database connection parameters from environment variables
    let db_host = std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "clocdb".to_string());
    let db_password = std::env::var("DATABASE_PASSWORD").ok();
    
    let connection_string = match db_password {
        Some(pwd) => format!("host={} user={} password={} dbname={}", db_host, db_user, pwd, db_name),
        None => format!("host={} user={} dbname={}", db_host, db_user, db_name),
    };
    
    tracing::info!("Connecting to database at host={}", db_host);
    
    let manager = PostgresConnectionManager::new_from_stringlike(
        &connection_string,
        NoTls,
    )
    .unwrap();
    let _config = tokio_postgres::Config::new()
        .dbname(&db_name)
        .host(&db_host)
        .user(&db_user)
        .to_owned();
    // let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();

    start_application(socket, pool.clone()).await;
}
