use std::collections::HashMap;

pub const ERROR: u8 = 1;

/// Estado de _request_ enviado a servicio externo
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// Solicitud enviada
    Sent,
    /// Solicitud exitosa
    Ok,
    /// Solicitud con errores
    Failed,
}

type TransactionAnswers = HashMap<String, Answer>;

/// Representación del paquete turístico a procesar
#[derive(Clone)]
pub struct Transaction {
    /// id único de la transacción
    pub id: String,
    /// precio del paquete
    pub precio: String,
}

pub struct EntityAnswer {
    pub entity_name: string,
    pub transaction_id: usize,
    pub answer: Answer
}

/// Representación de la respuesta del pago enviado
/// por parte de la Entidad Externa (*Aerolínea*, *Banco* u *Hotel*)
#[derive(Clone, Debug)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub success: bool,
}

pub struct ServerResponse {
    pub transaction_id: String,
    pub response: bool
}