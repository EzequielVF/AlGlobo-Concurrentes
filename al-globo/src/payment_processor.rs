use std::collections::HashMap;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, SyncContext};
use crate::external_entity::{SendTransaction, ExternalEntity, TransactionResult, TransactionResultMessage};
use crate::{Log, Logger, Writer};
use crate::logger::Log;
use crate::types::{EntityAnswer, Transaction};


/// Procesador de pagos
pub struct PaymentProcessor {
    /// Dirección de Actor *Banco*
    bank_addr: Addr<ExternalEntity>,
    /// Dirección de Actor *Aerolínea*
    airline_addr: Addr<ExternalEntity>,
    /// Dirección de Actor *Hotel*
    hotel_addr: Addr<ExternalEntity>,
    /// Dirección de Actor Banco
    logger_addr: Addr<Logger>,
    /// Dirección de Actor de escritura de fallas
    writer_addr: Addr<Writer>,
    /// Estado interno de las transacciones en proceso
    /// - clave: id transacción
    /// - valor: estado de los requests enviados a los servicios
    entity_answers: HashMap<usize, TransactionState>,

}

impl PaymentProcessor {
    pub fn new(
        airline_addr: Addr<ExternalEntity>, bank_addr: Addr<ExternalEntity>,hotel_addr: Addr<ExternalEntity>,
        writer_addr: Addr<Writer>, logger_addr: Addr<Logger>) -> Self {
        PaymentProcessor {
            bank_addr,
            airline_addr,
            hotel_addr,
            logger_addr,
            writer_addr,
            entity_answers: HashMap::<usize, HashMap<String, TransactionState>::new()>,
        }
    }
}

impl Actor for PaymentProcessor {
    type Context = SyncContext<Self>;
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTransactionToEntities(pub Transaction);

impl Handler<SendTransactionToEntities> for PaymentProcessor {
    type Result = ();

    fn handle(&mut self, msg: SendTransactionToEntities, ctx: &mut Context<Self>) -> Self::Result {
        let transaction = msg.0;

        self.airline_addr.do_send(SendTransaction((transaction).clone(), ctx.address()));
        self.bank_addr.do_send(SendTransaction((transaction).clone(), ctx.address()));
        self.hotel_addr.do_send(SendTransaction((transaction).clone(), ctx.address()));

        self.entity_answers.insert(transaction.id, HashMap::from([
                        ("BANK".to_string(), RequestState::Sent),
                        ("AIRLINE".to_string(), RequestState::Sent),
                        ("HOTEL".to_string(), RequestState::Sent),
        ]));

        self.logger_addr.do_send(Log(format!("Se envía paquete de id {} a entidades para procesamiento", transaction.id)));
    }
}

/// Mensaje para manejar la respuesta de los servicios externos
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessEntityAnswer(pub EntityAnswer);

impl Handler<ProcessEntityAnswer> for PaymentProcessor {
    type Result = ();

    fn handle(&mut self, msg: ProcessEntityAnswer, _ctx: &mut Context<Self>) -> Self::Result {
       let entity_answer = msg.0;

        if let Some(transaction_answers) = self.entity_answers.get_mut(&entity_answer.transaction_id) {

            transaction_answers.insert(entity_answer.entity_name, entity_answer.answer);

            self.logger_addr.do_send(
                Log(format!("Se registra respuesta de entidad {} para transacción de id {} con \
                resultado: {:?}", entity_answer.entity_name, entity_answer.transaction_id, entity_answer.answer)
            ));







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

                    self.airline_addr
                        .do_send(TransactionResultMessage(transaction_result.clone()));
                    self.bank_addr
                        .do_send(TransactionResultMessage(transaction_result.clone()));
                    self.hotel_addr
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
                                    self.airline_addr.do_send(TransactionResultMessage(
                                        transaction_result.clone(),
                                    ));
                                }
                                "BANK" => {
                                    self.bank_addr.do_send(TransactionResultMessage(
                                        transaction_result.clone(),
                                    ));
                                }
                                "HOTEL" => {
                                    self.hotel_addr.do_send(TransactionResultMessage(
                                        transaction_result.clone(),
                                    ));
                                }
                                _ => {
                                    self.logger_addr
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
