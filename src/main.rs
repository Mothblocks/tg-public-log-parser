use std::sync::Arc;

use axum::{http::StatusCode, Router};
use eyre::Context;

mod app_state;
mod parsers;
mod route;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("tg-public-log-parser");

    let state = Arc::new(app_state::AppState::load().context("loading app state")?);
    tracing::info!("hosting on {}", state.config.address);

    let listener = tokio::net::TcpListener::bind(state.config.address).await?;
    let app = Router::new()
        .route("/", axum::routing::get(route::get))
        .route("/{*path}", axum::routing::get(route::get))
        .route("/favicon.ico", axum::routing::get(|| async { "" }))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
