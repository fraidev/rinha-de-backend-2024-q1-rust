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

    let limite = app_state.limites.get(&id);

    let transacao = match tipo {
        TipoTransacao::Credito => {
            let tranasacao = sqlx::query!(
                "SELECT * FROM creditar($1, $2, $3)",
                id,
                login.valor as i32,
                login.descricao
            )
            .fetch_one(&app_state.db)
            .await
            .map_err(map_sql_error)?;
            (
                tranasacao.novo_saldo,
                tranasacao.possui_erro,
                tranasacao.mensagem,
            )
        }
        TipoTransacao::Debito => {
            let transacao = sqlx::query!(
                "SELECT * FROM debitar($1, $2, $3)",
                id,
                login.valor as i32,
                login.descricao
            )
            .fetch_one(&app_state.db)
            .await
            .map_err(map_sql_error)?;
            (
                transacao.novo_saldo,
                transacao.possui_erro,
                transacao.mensagem,
            )
        }
    };

    if transacao.1.is_some() && transacao.1.unwrap() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, transacao.2.unwrap()));
    }

    Ok(Json(json!({"limite": limite, "saldo": transacao.0})))
}

pub async fn extrato(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cliente = sqlx::query!("SELECT * FROM cliente WHERE id = $1", id)
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
        valor: t.valor as u32,
        tipo: t.tipo.clone(),
        descricao: t.descricao.clone(),
        realizada_em: t.realizada_em,
    })
    .collect();

    let extrato = Extrato {
        saldo: Saldo {
            total: cliente.saldo,
            data_extrato: Utc::now().naive_utc(),
            limite: cliente.limite as u32,
        },
        ultimas_transacoes,
    };

    Ok(Json(json!(extrato)))
}

fn map_sql_error(err: sqlx::Error) -> (StatusCode, String) {
    match err {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, err.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
