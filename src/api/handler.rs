use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

pub async fn transacoes(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    login: Json<EnviaTranasacao>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // validate
    match login.tipo.as_str() {
        "c" => {}
        "d" => {}
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Tipo invalido".to_string(),
            ))
        }
    }

    if login.descricao.len() > 20 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Descrição da transação muito longa".to_string(),
        ));
    }

    if login.descricao.len() == 0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Descrição da transação muito curta".to_string(),
        ));
    }




    let user = sqlx::query!("SELECT * FROM clientes WHERE id = $1", id)
        .fetch_one(&app_state.db)
        .await
        .map_err(map_sql_error)?;

    let transaction = match login.tipo.as_str() {
        "c" => {
            let a = sqlx::query!(
                "SELECT * FROM creditar($1, $2, $3) AS result",
                id,
                login.valor as i32,
                login.descricao
            )
            .fetch_one(&app_state.db)
            .await
            .map_err(map_sql_error)?;

            (a.novo_saldo, a.possui_erro, a.mensagem)
        }
        "d" => {
            let b = sqlx::query!(
                "SELECT * FROM debitar($1, $2, $3) AS result",
                id,
                login.valor as i32,
                login.descricao
            )
            .fetch_one(&app_state.db)
            .await
            .map_err(map_sql_error)?;

            (b.novo_saldo, b.possui_erro, b.mensagem)
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Tipo de transação inválido".to_string(),
            ))
        }
    };

    Ok(Json(json!({"limite": user.limite, "saldo": transaction.0})))
}

pub async fn extrato(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = sqlx::query!("select * from clientes where id = $1", id,)
        .fetch_one(&app_state.db)
        .await
        .map_err(map_sql_error)?;

    let ultimas_transacoes = sqlx::query!(
        "SELECT * FROM transacoes WHERE cliente_id = $1 ORDER BY realizada_em DESC LIMIT 10",
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

    let client_saldo = sqlx::query!("SELECT * FROM saldos WHERE cliente_id = $1", id)
        .fetch_one(&app_state.db)
        .await
        .map_err(map_sql_error)?;

    let extract = Extrato {
        saldo: Saldo {
            total: client_saldo.valor as i64,
            data_extrato: Utc::now().naive_utc(),
            limite: client.limite as u64,
        },
        ultimas_transacoes,
    };

    Ok(Json(json!(extract)))
}

#[derive(Deserialize)]
pub struct EnviaTranasacao {
    pub valor: u32,
    pub tipo: String,
    pub descricao: String,
}

#[derive(Serialize)]
pub struct Extrato {
    saldo: Saldo,
    ultimas_transacoes: Vec<Transacao>,
}

#[derive(Serialize)]
pub struct Saldo {
    total: i64,
    data_extrato: NaiveDateTime,
    limite: u64,
}

#[derive(Serialize)]
pub struct Transacao {
    valor: u64,
    tipo: String,
    descricao: String,
    realizada_em: NaiveDateTime,
}

fn map_sql_error(err: sqlx::Error) -> (StatusCode, String) {
    match err {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, err.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
