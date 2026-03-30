use axum::Router;
use axum::middleware::from_fn_with_state;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::{cors::{CorsLayer, Any}, trace::TraceLayer};
use dotenvy::dotenv;

mod config;
mod errors;
mod models;
mod routes;
mod middleware;
mod blockchain;
mod db;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or("z4_backend=debug".to_string())
        )
        .init();

    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .expect("Gagal konek ke Supabase");

    tracing::info!("Koneksi Supabase berhasil");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Gagal jalankan migrations");

    tracing::info!("Migrations selesai");

    let state = Arc::new(AppState { db, config: config.clone() });

    blockchain::listener::start_event_listener(state.clone()).await;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let public_routes = Router::new()
        .merge(routes::auth::router())
        .merge(routes::plot::router());

    let protected_routes = Router::new()
        .merge(routes::token_sale::router())
        .merge(routes::allocation::router())
        .merge(routes::portfolio::router())
        .layer(from_fn_with_state(
            state.clone(),
            crate::middleware::auth::require_auth,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Z4 backend jalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}