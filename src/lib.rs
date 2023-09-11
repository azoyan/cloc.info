#![feature(byte_slice_trim_ascii)]
#![feature(async_closure)]

pub mod application;
pub mod cloner;
pub mod providers;
pub mod repository;
pub mod server;
pub mod statistic;
pub mod websocket;

pub const KB: u64 = 1024;
pub const MB: u64 = 1024 * KB;
pub const GB: u64 = 1024 * MB;

type DbId = i64;
