//! Hand-written home page for terminal browsers (Lynx, w3m, Links, curl).
//!
//! The regular home page is a Vite/pug single-page application: its layout comes
//! from CSS, its behaviour from JavaScript and its markup repeats the site
//! navigation. In text mode all of that turns into noise, so terminal clients get
//! this small standalone document instead of the app shell.

use axum::{
    body::Body,
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
};
use chrono::{Datelike, Utc};

const INDEX_HTML: &str = include_str!("terminal/index.html");
const START_YEAR: i32 = 2024;

/// Same copyright line as the regular footer, so both pages cannot diverge.
fn copyright_years() -> String {
    let current = Utc::now().year();
    if current > START_YEAR {
        format!("{START_YEAR}-{current}")
    } else {
        START_YEAR.to_string()
    }
}

pub fn index_page() -> Response<Body> {
    let html = INDEX_HTML.replace("%COPYRIGHT_YEARS%", &copyright_years());

    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        )],
        html,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::INDEX_HTML;

    #[test]
    fn terminal_page_keeps_the_form_contract() {
        assert!(INDEX_HTML.contains("<form method=\"get\" action=\"/count\">"));
        assert!(INDEX_HTML.contains("name=\"url\""));
        assert!(INDEX_HTML.contains("type=\"submit\""));
    }

    #[test]
    fn terminal_page_needs_no_styles_and_no_scripts() {
        assert!(!INDEX_HTML.contains("<script"));
        assert!(!INDEX_HTML.contains("stylesheet"));
        assert!(!INDEX_HTML.contains("class="));
    }
}
