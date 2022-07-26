#![feature(byte_slice_trim_ascii)]
pub mod cloner;
pub mod github;
pub mod providers;
pub mod repository;
pub mod server;
pub mod websocket;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;

type DbId = i64;
