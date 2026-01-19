mod domain;
mod infrastructure;
mod middleware;
mod api;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use infrastructure::{
    database::{create_pool, run_migrations},
    repositories::*,
    session_cache::SessionCache,
};
use api::{create_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rabbit_todo_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://todos.db".to_string());
    
    tracing::info!("Connecting to database: {}", database_url);
    let pool = create_pool(&database_url).await?;
    
    tracing::info!("Running migrations");
    run_migrations(&pool).await?;

    // Initialize repositories
    let user_repo = Arc::new(SqliteUserRepository::new(pool.clone()));
    let group_repo = Arc::new(SqliteGroupRepository::new(pool.clone()));
    let todo_repo = Arc::new(SqliteTodoRepository::new(pool.clone()));
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    // Initialize session cache (1000 max sessions, 24 hour TTL)
    let session_cache = SessionCache::new(1000, 86400);

    // Create app state
    let state = AppState {
        user_repo,
        group_repo,
        todo_repo,
        session_repo,
        session_cache,
    };

    // Create router
    let app = create_router(state);

    // Start server
    let addr = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    
    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
