//! Static HTTP server that serves the bundled Flutter web build (the mobile
//! deck UI). Embedded via `rust-embed` at compile time so the agent ships as
//! a single executable — no separate Python server needed.
//!
//! Build prerequisite: run `flutter build web --release` in `mobile/` before
//! `cargo build`. If the bundle is empty (e.g. a developer skipped the step),
//! the server logs a warning at startup and exits gracefully.

use anyhow::{Context, Result};
use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use rust_embed::Embed;
use std::net::SocketAddr;

#[derive(Embed)]
#[folder = "../../mobile/build/web/"]
struct WebAssets;

pub async fn start(port: u16) -> Result<SocketAddr> {
    if WebAssets::iter().next().is_none() {
        anyhow::bail!(
            "Flutter web bundle is empty. Run `flutter build web --release` in mobile/ first."
        );
    }

    let app = Router::new().fallback(any(handler));

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {}", addr))?;
    let bound = listener.local_addr()?;
    tracing::info!(addr = %bound, "static web server listening");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!(error = ?e, "axum server failed");
        }
    });

    Ok(bound)
}

async fn handler(uri: Uri) -> Response {
    let raw_path = uri.path().trim_start_matches('/');
    let path = if raw_path.is_empty() {
        "index.html"
    } else {
        raw_path
    };

    if let Some(content) = WebAssets::get(path) {
        return build_response(path, content.data.into_owned());
    }

    // SPA fallback: serve index.html so client-side routes resolve.
    match WebAssets::get("index.html") {
        Some(content) => build_response("index.html", content.data.into_owned()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn build_response(path: &str, bytes: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
