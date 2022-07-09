use crate::communication::{Commit, Error, Pay, Rollback, Successful, Unknown};

pub const ERROR: u8 = 1;

/// Estado de _request_ enviado a servicio externo
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// Solicitud enviada
    Pending,
    /// Solicitud exitosa
    Ok,
    /// Solicitud con errores
    Failed,
}

//type TransactionAnswers = HashMap<String, Answer>;

/// Representación del paquete turístico a procesar
#[derive(Clone)]
pub struct Transaction {
    /// id único de la transacción
    pub id: String,
    /// precio del paquete
    pub precio: String,
}

#[derive(Debug)]
pub struct EntityAnswer {
    pub entity_name: String,
    pub transaction_id: String,
    pub answer: Answer,
}

/// Representación de la respuesta del pago enviado
/// por parte de la Entidad Externa (*Aerolínea*, *Banco* u *Hotel*)
#[derive(Clone, Debug)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub success: bool,
}

#[derive(Debug)]
pub struct ServerResponse {
    pub transaction_id: String,
    pub response: bool,
}

pub enum Type {
    Error,
    Pay,
    Successful,
    Commit,
    Rollback,
    Unknown,
}

impl From<u8> for Type {
    fn from(code: u8) -> Type {
        match code & 0xF0 {
            0x00 => Pay,
            0x10 => Successful,
            0x20 => Error,
            0x30 => Commit,
            0x40 => Rollback,
            _ => Unknown,
        }
    }
}


impl From<Type> for u8 {
    fn from(code: Type) -> u8 {
        match code {
            Pay => 0x00,
            Successful => 0x10,
            Error => 0x20,
            Commit => 0x30,
            Rollback => 0x40,
            _ => 0x99,
        }
    }
}
