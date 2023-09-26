use axum::{
    extract::{Path, State},
    response::Response,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::{DateTime, Utc};
use hyper::{header::CONTENT_TYPE, Body, StatusCode};
use mime_guess::mime::APPLICATION_JSON;
use multimap::MultiMap;
use serde_json::json;
use tokio_postgres::NoTls;

pub async fn largest(
    Path(limit): Path<i64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();
    let result = pool
        .query(
            "select * from all_view order by size desc limit $1",
            &[&limit],
        )
        .await;

    match result {
        Ok(rows) => {
            let mut map = MultiMap::with_capacity(rows.len());

            for row in rows {
                let hostname: String = row.get("hostname");
                let owner: String = row.get("owner");
                let repository_name: String = row.get("repository_name");
                let branch: String = row.get("name");
                let size: i64 = row.get("size");
                let value = json!({
                    "hostname": hostname,
                    "owner": owner,
                    "repository_name": repository_name,
                    "branch_name": branch,
                    "size": size,
                });
                map.insert(repository_name, value);
                if map.keys().len() == limit as usize {
                    break;
                }
            }
            let json = serde_json::to_string(&map).unwrap();
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
    Path(limit): Path<i64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();

    let result = pool
        .query("select * from all_view order by time desc;", &[])
        .await;

    match result {
        Ok(rows) => {
            let mut map = MultiMap::with_capacity(rows.len());

            for row in rows {
                let hostname: String = row.get("hostname");
                let owner: String = row.get("owner");
                let repository_name: String = row.get("repository_name");
                let branch: String = row.get("name");
                let time: DateTime<Utc> = row.get("time");
                let value = json!({
                    "hostname": hostname,
                    "owner": owner,
                    "repository_name": repository_name,
                    "branch_name": branch,
                    "time": time.to_rfc3339(),
                });
                map.insert(repository_name, value);
                if map.keys().len() == limit as usize {
                    break;
                }
            }
            let json = serde_json::to_string(&map).unwrap();
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
    Path(limit): Path<i64>,
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let pool = connection_pool.get().await.unwrap();

    let result = pool
        .query("select * from popular_repositories limit $1", &[&limit])
        .await;

    match result {
        Ok(rows) => {
            let mut map = MultiMap::with_capacity(rows.len());

            for row in rows {
                let hostname: String = row.get("hostname");
                let owner: String = row.get("owner");
                let repository_name: String = row.get("repository_name");
                let branch: String = row.get("name");
                let count: i64 = row.get("count");
                let value = json!({
                    "hostname": hostname,
                    "owner": owner,
                    "repository_name": repository_name,
                    "branch_name": branch,
                    "count": count,
                });
                map.insert(repository_name, value);
                if map.keys().len() == limit as usize {
                    break;
                }
            }
            let json = serde_json::to_string(&map).unwrap();
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
