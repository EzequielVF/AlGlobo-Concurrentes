use std::collections::HashMap;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};

use crate::external_entity::{
    ExtEntNewPayment, ExternalEntity, TransactionResult, TransactionResultMessage,
};
use crate::{Log, Logger};

/// Representación del paquete turístico a procesar
#[derive(Clone)]
pub struct TouristPackage {
    /// id único de la transacción
    pub id: usize,
    /// precio del paquete
    pub precio: usize,
}

/// Estado de una transacción
/// - clave: nombre del servicio
/// - valor: estado actual del _request_
type TransactionState = HashMap<String, RequestState>;

/// Procesador de pagos
pub struct PaymentProcessor {
    /// Dirección de Actor *Banco*
    bank_address: Addr<ExternalEntity>,
    /// Dirección de Actor *Aerolínea*
    airline_address: Addr<ExternalEntity>,
    /// Dirección de Actor *Hotel*
    hotel_address: Addr<ExternalEntity>,
    /// Estado interno de las transacciones en proceso
    /// - clave: id transacción
    /// - valor: estado de los requests enviados a los servicios
    entity_answers: HashMap<usize, TransactionState>,
    /// Dirección de Actor Banco
    logger_address: Addr<Logger>,
}

/// Estado de _request_ enviado a servicio externo
#[derive(Debug, Clone, PartialEq)]
pub enum RequestState {
    /// Solicitud enviada
    Sent,
    /// Solicitud exitosa
    Ok,
    /// Solicitud con errores
    Failed,
}

impl PaymentProcessor {
    pub fn new(
        bank_address: Addr<ExternalEntity>,
        airline_address: Addr<ExternalEntity>,
        hotel_address: Addr<ExternalEntity>,
        logger: Addr<Logger>,
    ) -> Self {
        PaymentProcessor {
            bank_address,
            airline_address,
            hotel_address,
            entity_answers: HashMap::<usize, HashMap<String, RequestState>>::new(),
            logger_address: logger,
        }
    }
}

impl Actor for PaymentProcessor {
    type Context = Context<Self>;
}

/// Mensaje para procesar el pago de un nuevo Paquete Turístico
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct PayProcNewPayment(pub TouristPackage);

impl Handler<PayProcNewPayment> for PaymentProcessor {
    type Result = ();

    fn handle(&mut self, msg: PayProcNewPayment, ctx: &mut Context<Self>) -> Self::Result {
        self.airline_address
            .do_send(ExtEntNewPayment((msg.0).clone(), ctx.address()));
        self.bank_address
            .do_send(ExtEntNewPayment((msg.0).clone(), ctx.address()));
        self.hotel_address
            .do_send(ExtEntNewPayment((msg.0).clone(), ctx.address()));

        self.entity_answers.insert(
            msg.0.id,
            HashMap::from([
                ("BANK".to_string(), RequestState::Sent),
                ("AIRLINE".to_string(), RequestState::Sent),
                ("HOTEL".to_string(), RequestState::Sent),
            ]),
        );
        self.logger_address.do_send(Log(format!(
            "Se envía paquete de id {} a entidades para procesamiento",
            msg.0.id
        )));
    }
}

/// Mensaje para manejar la respuesta de los servicios externos
#[derive(Message)]
#[rtype(result = "()")]
pub struct EntityAnswer(pub String, pub usize, pub RequestState);

impl Handler<EntityAnswer> for PaymentProcessor {
    type Result = ();

    fn handle(&mut self, msg: EntityAnswer, _ctx: &mut Context<Self>) -> Self::Result {
        let transaction_id = &msg.1;
        if let Some(transaction) = self.entity_answers.get_mut(transaction_id) {
            transaction.insert(msg.0.clone(), msg.2.clone());
            self.logger_address
                .do_send(Log(format!("[{}-ID:{}-Estado:{:?}]", msg.0, msg.1, msg.2)));

            if transaction
                .iter()
                .all(|(_n, state)| *state != RequestState::Sent)
            {
                if transaction
                    .iter()
                    .all(|(_n, state)| *state == RequestState::Ok)
                {
                    let transaction_result = TransactionResult {
                        transaction_id: transaction_id.to_string(),
                        result: true,
                    };

                    self.airline_address
                        .do_send(TransactionResultMessage(transaction_result.clone()));
                    self.bank_address
                        .do_send(TransactionResultMessage(transaction_result.clone()));
                    self.hotel_address
                        .do_send(TransactionResultMessage(transaction_result));
                } else {
                    let transaction_result: TransactionResult = TransactionResult {
                        transaction_id: transaction_id.to_string(),
                        result: false,
                    };
                    for (service, state) in transaction.iter() {
                        if *state == RequestState::Ok {
                            match service.as_str() {
                                "AIRLINE" => {
                                    self.airline_address.do_send(TransactionResultMessage(
                                        transaction_result.clone(),
                                    ));
                                }
                                "BANK" => {
                                    self.bank_address.do_send(TransactionResultMessage(
                                        transaction_result.clone(),
                                    ));
                                }
                                "HOTEL" => {
                                    self.hotel_address.do_send(TransactionResultMessage(
                                        transaction_result.clone(),
                                    ));
                                }
                                _ => {
                                    self.logger_address
                                        .do_send(Log("No debería suceder esto".to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
