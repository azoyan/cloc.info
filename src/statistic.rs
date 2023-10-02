use crate::logic::info::{LargestRepositories, PopularRepositories, RecentRepositories};
use axum::{
    extract::{Path, State},
    response::Response,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use hyper::{header::CONTENT_TYPE, Body, StatusCode};
use mime_guess::mime::APPLICATION_JSON;
use tokio_postgres::NoTls;

pub async fn largest(
    Path(limit): Path<i64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();
    let result = pool.query("select * from all_view;", &[]).await;

    match result {
        Ok(rows) => {
            let largest = LargestRepositories::from(rows).top(limit as usize);
            let json = serde_json::to_string(&largest).unwrap();
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(json))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}

pub async fn recent(
    Path(limit): Path<u64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();

    let result = pool.query("select * from all_view;", &[]).await;

    match result {
        Ok(rows) => {
            let recent = RecentRepositories::from(rows).top(limit as usize);
            let json = serde_json::to_string(&recent).unwrap();
            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(json))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}

pub async fn popular(
    Path(limit): Path<u64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();

    let result = pool.query("select * from popular_repositories;", &[]).await;

    match result {
        Ok(rows) => {
            let popular = PopularRepositories::from(rows).top(limit as usize);
            let json = serde_json::to_string(&popular).unwrap();

            Response::builder()
                .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
                .body(Body::from(json))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}
