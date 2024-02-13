use std::sync::Arc;
use std::time::Duration;

use api::router;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;

pub mod api;

pub struct AppState {
    db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::fmt::init();

    // get environment variables
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    tracing::info!("Connecting to database at {}", db_connection_str);

    // set up connection pool
    let pool = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(1000)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    // build our application with a route
    let app_state_arc = Arc::new(AppState { db: pool });
    let app = router::create_router(app_state_arc);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
