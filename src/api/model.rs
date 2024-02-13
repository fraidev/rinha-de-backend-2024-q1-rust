use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EnviaTranasacao {
    pub valor: u32,
    pub tipo: String,
    pub descricao: String,
}

#[derive(Serialize)]
pub struct Extrato {
    pub saldo: Saldo,
    pub ultimas_transacoes: Vec<Transacao>,
}

#[derive(Serialize)]
pub struct Saldo {
    pub total: i32,
    pub data_extrato: NaiveDateTime,
    pub limite: u32,
}

#[derive(Serialize)]
pub struct Transacao {
    pub valor: u32,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: NaiveDateTime,
}

pub enum TipoTransacao {
    Credito,
    Debito,
}
