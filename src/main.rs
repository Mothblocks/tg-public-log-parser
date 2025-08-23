use std::sync::Arc;

use axum::Router;
use eyre::Context;
use tracing_subscriber::prelude::*;

mod app_state;
mod ongoing_round_protection;
mod parsers;
mod route;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!("tg-public-log-parser");

    let state = Arc::new(
        app_state::AppState::load()
            .await
            .context("loading app state")?,
    );
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
