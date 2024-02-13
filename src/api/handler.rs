use super::model::{EnviaTranasacao, Extrato, Saldo, TipoTransacao, Transacao};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

pub async fn transacoes(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    login: Json<EnviaTranasacao>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // validate
    let tipo = match login.tipo.as_str() {
        "c" => TipoTransacao::Credito,
        "d" => TipoTransacao::Debito,
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Tipo invalido".to_string(),
            ))
        }
    };

    match login.descricao.len() {
        1..=20 => (),
        0 => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Descrição da transação vazia".to_string(),
            ))
        }
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Descrição da transação muito longa".to_string(),
            ))
        }
    };

    let user = sqlx::query!("SELECT * FROM cliente WHERE id = $1", id)
        .fetch_one(&app_state.db)
        .await
        .map_err(map_sql_error)?;

    let transaction = match tipo {
        TipoTransacao::Credito => {
            let creditar_result = sqlx::query!(
                "SELECT * FROM creditar($1, $2, $3) AS result",
                id,
                login.valor as i32,
                login.descricao
            )
            .fetch_one(&app_state.db)
            .await
            .map_err(map_sql_error)?;

            (
                creditar_result.novo_saldo,
                creditar_result.possui_erro,
                creditar_result.mensagem,
            )
        }
        TipoTransacao::Debito => {
            let debitar_result = sqlx::query!(
                "SELECT * FROM debitar($1, $2, $3) AS result",
                id,
                login.valor as i32,
                login.descricao
            )
            .fetch_one(&app_state.db)
            .await
            .map_err(map_sql_error)?;

            (
                debitar_result.novo_saldo,
                debitar_result.possui_erro,
                debitar_result.mensagem,
            )
        }
    };

    Ok(Json(json!({"limite": user.limite, "saldo": transaction.0})))
}

pub async fn extrato(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = sqlx::query!("SELECT * FROM cliente WHERE id = $1", id)
        .fetch_one(&app_state.db)
        .await
        .map_err(map_sql_error)?;

    let ultimas_transacoes = sqlx::query!(
        "SELECT * FROM transacao WHERE cliente_id = $1 ORDER BY realizada_em DESC LIMIT 10",
        id
    )
    .fetch_all(&app_state.db)
    .await
    .map_err(map_sql_error)?
    .iter()
    .map(|t| Transacao {
        valor: t.valor as u64,
        tipo: t.tipo.clone(),
        descricao: t.descricao.clone(),
        realizada_em: t.realizada_em,
    })
    .collect();

    let extract = Extrato {
        saldo: Saldo {
            total: client.valor as i64,
            data_extrato: Utc::now().naive_utc(),
            limite: client.limite as u64,
        },
        ultimas_transacoes,
    };

    Ok(Json(json!(extract)))
}

fn map_sql_error(err: sqlx::Error) -> (StatusCode, String) {
    match err {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, err.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
