use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn transacoes(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    login: Json<EnviaTranasacao>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = sqlx::query!("select * from clientes where id = $1", id,)
        .fetch_one(&app_state.db)
        .await
        .map_err(not_found)?;

    tracing::info!("Logging in user: {}", id);
    tracing::info!("valor: {}", login.valor);
    Ok(Json(json!({"limite": 10000, "saldo": 1000})))
}

pub async fn extrato(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = sqlx::query!("select * from clientes where id = $1", id,)
        .fetch_one(&app_state.db)
        .await
        .map_err(not_found)?;

    Ok(Json(json!({"limite": 10000, "saldo": 1000})))
}

#[derive(Deserialize)]
pub struct EnviaTranasacao {
    pub valor: u64,
    pub tipo: String,
    pub descricao: String,
}

// #[derive(Deserialize)]
pub struct Extrato {
    saldo: Saldo,
    ultimas_transacoes: Vec<Transacao>,
}

// #[derive(Deserialize)]
pub struct Saldo {
    total: i64,
    data_extrato: DateTime<Utc>,
    limite: u64,
}

// #[derive(Deserialize)]
pub struct Transacao {
    valor: u64,
    tipo: String,
    descricao: String,
    realizada_em: DateTime<Utc>,
}
/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn not_found<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::NOT_FOUND, err.to_string())
}
