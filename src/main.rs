use axum::{routing::get, Router};
use eyre::Context;
use http::Method;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
        ]);

    let app = Router::new()
        .without_v07_checks()
        .route("/", get(route::get))
        .route("/favicon.ico", get(|| async { "" }))
        .route("/*path", get(route::get))
        .layer(cors)
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
