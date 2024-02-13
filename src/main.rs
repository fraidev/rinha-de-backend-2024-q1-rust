use std::collections::HashMap;
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
    limites: HashMap<i32, i32>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // get environment variables
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    // set up connection pool
    let pool = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(1000)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    let clients = sqlx::query!("SELECT * FROM cliente")
        .fetch_all(&pool)
        .await
        .expect("can't fetch clients");

    // As limits are never changed, we can use a cache
    let mut limites = HashMap::new();
    for cliente in clients {
        limites.insert(cliente.id, cliente.limite as i32);
    }

    // build our application with a route
    let app_state_arc = Arc::new(AppState {
        db: pool,
        limites,
    });
    let app = router::create_router(app_state_arc);

    // run app
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
