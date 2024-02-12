use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use super::handler::{extrato, transacoes};
use crate::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    return Router::new()
        .route("/clientes/:id/transacoes", post(transacoes))
        .route("/clientes/:id/extrato", get(extrato))
        .with_state(app_state);
}
