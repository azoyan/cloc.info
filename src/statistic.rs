use crate::logic::info::{
    to_repository_path, LargestRepositories, PopularRepositories, RecentRepositories,
};
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

const SITE_URL: &str = "https://cloc.info";
const SITEMAP_REPOSITORY_LIMIT: i64 = 200;

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

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn sitemap_entry(location: &str, changefreq: &str, priority: &str) -> String {
    format!(
        "  <url>\n    <loc>{}</loc>\n    <changefreq>{changefreq}</changefreq>\n    <priority>{priority}</priority>\n  </url>\n",
        escape_xml(location)
    )
}

fn is_placeholder_repository(repository_name: &str) -> bool {
    const PLACEHOLDERS: [&str; 12] = [
        "test", "tests", "testing", "dummy", "example", "sample", "foo", "bar", "baz", "temp",
        "tmp", "demo",
    ];

    let name = repository_name
        .trim_end_matches(".git")
        .to_ascii_lowercase();
    let name = name.trim_end_matches(|character: char| {
        character.is_ascii_digit() || matches!(character, '-' | '_' | '.')
    });

    PLACEHOLDERS.contains(&name)
}

/// Lists the pages that should be indexed: the home page and the most viewed
/// repository reports. Branch pages are `noindex`, so they are left out.
fn sitemap_xml(repositories: &[(String, String, String)]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    xml.push_str(&sitemap_entry(&format!("{SITE_URL}/"), "daily", "1.0"));

    let mut seen = std::collections::HashSet::new();
    for (hostname, owner, repository_name) in repositories {
        if is_placeholder_repository(repository_name) {
            continue;
        }
        let url = format!(
            "{SITE_URL}/{}",
            to_repository_path(hostname, owner, repository_name.trim_end_matches(".git"))
        );
        // The database can hold `axum`, `axum.git` and `AXUM` as separate rows;
        // they all resolve to the same page, so list the URL only once.
        if seen.insert(url.to_ascii_lowercase()) {
            xml.push_str(&sitemap_entry(&url, "weekly", "0.5"));
        }
    }

    xml.push_str("</urlset>\n");
    xml
}

pub async fn sitemap(
    State(connection_pool): State<Pool<PostgresConnectionManager<NoTls>>>,
) -> Response<Body> {
    let repositories = match connection_pool.get().await {
        Ok(pool) => match pool
            .query(
                "select hostname, owner, repository_name, sum(count) as total
                 from popular_repositories
                 group by hostname, owner, repository_name
                 order by total desc
                 limit $1",
                &[&SITEMAP_REPOSITORY_LIMIT],
            )
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| {
                    (
                        row.get::<&str, String>("hostname"),
                        row.get::<&str, String>("owner"),
                        row.get::<&str, String>("repository_name"),
                    )
                })
                .collect::<Vec<_>>(),
            Err(error) => {
                tracing::warn!("Can't build sitemap: {error}");
                Vec::new()
            }
        },
        Err(error) => {
            tracing::warn!("Can't build sitemap: {error}");
            Vec::new()
        }
    };

    (
        [
            ("Cache-Control", "public, max-age=3600"),
            ("Content-Type", "application/xml; charset=utf-8"),
        ],
        sitemap_xml(&repositories),
    )
        .into_response()
}

pub fn robots_txt() -> String {
    format!("User-agent: *\nAllow: /\nDisallow: /api/\nDisallow: /ws/\n\nSitemap: {SITE_URL}/sitemap.xml\n")
}

#[cfg(test)]
mod tests {
    use super::{robots_txt, sitemap_xml};

    fn repository(host: &str, owner: &str, name: &str) -> (String, String, String) {
        (host.to_string(), owner.to_string(), name.to_string())
    }

    #[test]
    fn sitemap_lists_the_home_page_and_repositories() {
        let xml = sitemap_xml(&[repository("github.com", "tokio-rs", "axum.git")]);

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<loc>https://cloc.info/</loc>"));
        assert!(xml.contains("<loc>https://cloc.info/github.com/tokio-rs/axum</loc>"));
        assert!(!xml.contains(".git"));
        assert!(xml.ends_with("</urlset>\n"));
    }

    #[test]
    fn sitemap_skips_repositories_that_resolve_to_the_same_url() {
        let xml = sitemap_xml(&[
            repository("github.com", "tokio-rs", "axum"),
            repository("github.com", "tokio-rs", "axum.git"),
            repository("github.com", "tokio-rs", "AXUM"),
        ]);

        assert_eq!(xml.matches("<loc>").count(), 2);
    }

    #[test]
    fn sitemap_skips_placeholder_repositories() {
        let xml = sitemap_xml(&[
            repository("github.com", "dummy", "test"),
            repository("github.com", "someone", "Test42.git"),
            repository("github.com", "tokio-rs", "axum"),
        ]);

        assert_eq!(xml.matches("<loc>").count(), 2);
        assert!(xml.contains("/github.com/tokio-rs/axum"));
    }

    #[test]
    fn sitemap_keeps_the_gitflic_project_prefix() {
        let xml = sitemap_xml(&[repository("gitflic.ru", "ivan", "sub")]);

        assert!(xml.contains("<loc>https://cloc.info/gitflic.ru/project/ivan/sub</loc>"));
    }

    #[test]
    fn sitemap_escapes_xml() {
        let xml = sitemap_xml(&[repository("github.com", "a&b", "c<d")]);

        assert!(xml.contains("https://cloc.info/github.com/a&amp;b/c&lt;d"));
    }

    #[test]
    fn robots_points_to_the_sitemap() {
        let robots = robots_txt();

        assert!(robots.contains("Disallow: /api/"));
        assert!(robots.contains("Sitemap: https://cloc.info/sitemap.xml"));
    }
}
