use std::collections::HashMap;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, SyncContext};
use crate::{Logger, Writer};
use crate::external_entity_sender::{ExternalEntitySender, SendTransactionToServer, SendConfirmationOrRollbackToServer};
use crate::logger::Log;
use crate::types::{Answer, EntityAnswer, Transaction, TransactionResult};
use crate::writer::WriteTransaction;


/// Procesador de pagos
pub struct TransactionManager {
    /// Dirección de Actor *Banco*
    bank_addr: Addr<ExternalEntitySender>,
    /// Dirección de Actor *Aerolínea*
    airline_addr: Addr<ExternalEntitySender>,
    /// Dirección de Actor *Hotel*
    hotel_addr: Addr<ExternalEntitySender>,
    /// Dirección de Actor Banco
    logger_addr: Addr<Logger>,
    /// Dirección de Actor de escritura de fallas
    writer_addr: Addr<Writer>,
    /// Estado interno de las transacciones en proceso
    /// - clave: id transacción
    /// - valor: estado de los requests enviados a los servicios
    entity_answers: HashMap<String, HashMap<String, Answer>>,
    transactions: HashMap<String, Transaction>

}

impl TransactionManager {
    pub fn new(
        airline_addr: Addr<ExternalEntitySender>, bank_addr: Addr<ExternalEntitySender>,hotel_addr: Addr<ExternalEntitySender>,
        writer_addr: Addr<Writer>, logger_addr: Addr<Logger>) -> Self {
        TransactionManager {
            bank_addr,
            airline_addr,
            hotel_addr,
            logger_addr,
            writer_addr,
            entity_answers: HashMap::<String, HashMap<String, Answer>>::new(),
            transactions: HashMap::<String, Transaction>::new()
        }
    }
}

impl Actor for TransactionManager {
    type Context = SyncContext<Self>;
}

/// Mensaje para enviar transacción a servicios externos
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTransactionToEntities(pub Transaction);

impl Handler<SendTransactionToEntities> for TransactionManager {
    type Result = ();

    fn handle(&mut self, msg: SendTransactionToEntities, ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction = msg.0;
        self.transactions.insert(transaction.id.clone(), transaction.clone());

        self.airline_addr.do_send(SendTransactionToServer((transaction).clone(), ctx.address()));
        self.bank_addr.do_send(SendTransactionToServer((transaction).clone(), ctx.address()));
        self.hotel_addr.do_send(SendTransactionToServer((transaction).clone(), ctx.address()));

        self.entity_answers.insert(transaction.id.clone(), HashMap::from([
                        ("BANK".to_string(), Answer::Pending),
                        ("AIRLINE".to_string(), Answer::Pending),
                        ("HOTEL".to_string(), Answer::Pending),
        ]));
        self.logger_addr.do_send(Log(format!("Se envía paquete de id {} a entidades para procesamiento", transaction.id)));
    }
}


/// Mensaje para manejar la respuesta de los servicios externos/// Mensaje para procesar respuesta de servicios externos
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessEntityAnswer(pub EntityAnswer);

impl Handler<ProcessEntityAnswer> for TransactionManager {
    type Result = ();

    fn handle(&mut self, msg: ProcessEntityAnswer, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let entity_answer = msg.0;

        if let Some(transaction_answers) = self.entity_answers.get_mut(&entity_answer.transaction_id) {
            transaction_answers.insert(entity_answer.entity_name.clone(), entity_answer.answer.clone());

            self.logger_addr.do_send(
                Log(format!("Se registra respuesta de entidad {} para transacción de id {} con \
                resultado: {:?}", entity_answer.entity_name, entity_answer.transaction_id, entity_answer.answer)
                ));


            let completed: bool = transaction_answers.iter().all(|(_n, state)| *state != Answer::Pending);

            if completed {
                let ok_entities = transaction_answers.iter().filter(|(_n, state)| *state == &Answer::Ok);

                let mut transaction_result = TransactionResult {
                    transaction_id: entity_answer.transaction_id.clone(),
                    success: true,
                };
                let mut message_type = "commit";

                    let has_fails: bool = transaction_answers.iter().all(|(_n, state)| *state !=Answer::Ok);

                if has_fails {
                    transaction_result.success = false;
                    message_type = "rollback";
                    if let Some(transaction) = self.transactions.get(entity_answer.transaction_id.as_str()) {
                        self.writer_addr.do_send(WriteTransaction(transaction.clone()));
                        self.logger_addr.do_send(Log(format!("Se escribe transacción de id {} en archivo de fallas",transaction.id)));
                    }

                }

                ok_entities.for_each(|(entity_name, Answer)| {
                    match entity_name.as_str() {
                        "AIRLINE" => { self.airline_addr.do_send(SendConfirmationOrRollbackToServer(transaction_result.clone())); }
                        "BANK" => { self.bank_addr.do_send(SendConfirmationOrRollbackToServer(transaction_result.clone())); }
                        "HOTEL" => { self.hotel_addr.do_send(SendConfirmationOrRollbackToServer(transaction_result.clone())); }
                        _ => { }
                    };
                    self.logger_addr.do_send(Log(format!("Se envía mensage de {} para la transaccion {} a entidad {} para procesamiento", message_type, entity_answer.transaction_id, entity_name)));
                })
            }
        }
    }
}
