use crate::logic::info::{LargestRepositories, PopularRepositories, RecentRepositories};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use hyper::{header::CONTENT_TYPE, StatusCode};
use mime_guess::mime::APPLICATION_JSON;
use serde::Serialize;
use std::fmt::Display;
use tokio_postgres::NoTls;

fn internal_server_error_response(error: impl Display) -> Response<Body> {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
}

fn json_response<T: Serialize>(payload: &T) -> Response<Body> {
    match serde_json::to_vec(payload) {
        Ok(json) => ([(CONTENT_TYPE, APPLICATION_JSON.essence_str())], json).into_response(),
        Err(error) => internal_server_error_response(error),
    }
}

pub async fn largest(
    Path(limit): Path<i64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = match connection_pool.get().await {
        Ok(pool) => pool,
        Err(error) => return internal_server_error_response(error),
    };
    let result = pool.query("select * from all_view;", &[]).await;

    match result {
        Ok(rows) => {
            let largest = LargestRepositories::from(rows).top(limit as usize);
            json_response(&largest)
        }
        Err(error) => internal_server_error_response(error),
    }
}

pub async fn recent(
    Path(limit): Path<u64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = match connection_pool.get().await {
        Ok(pool) => pool,
        Err(error) => return internal_server_error_response(error),
    };

    let result = pool.query("select * from all_view;", &[]).await;

    match result {
        Ok(rows) => {
            let recent = RecentRepositories::from(rows).top(limit as usize);
            json_response(&recent)
        }
        Err(error) => internal_server_error_response(error),
    }
}

pub async fn popular(
    Path(limit): Path<u64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = match connection_pool.get().await {
        Ok(pool) => pool,
        Err(error) => return internal_server_error_response(error),
    };

    let result = pool.query("select * from popular_repositories;", &[]).await;

    match result {
        Ok(rows) => {
            let popular = PopularRepositories::from(rows).top(limit as usize);
            json_response(&popular)
        }
        Err(error) => internal_server_error_response(error),
    }
}
