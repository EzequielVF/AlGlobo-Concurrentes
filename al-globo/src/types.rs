use std::collections::HashMap;

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
    pub id: usize,
    /// precio del paquete
    pub precio: usize,
}

pub struct EntityAnswer {
    pub entity_name: string,
    pub transaction_id: usize,
    pub answer: Answer
}