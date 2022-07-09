use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, Message, SyncContext};

use crate::{Logger, Writer};
use crate::logger::Log;
use crate::sender::{SendConfirmationOrRollback, Sender, SendTransaction};
use crate::types::{Answer, EntityAnswer, Transaction, TransactionResult};
use crate::writer::WriteTransaction;

pub struct TransactionManager {
    bank: Option<Addr<Sender>>,
    airline: Option<Addr<Sender>>,
    hotel: Option<Addr<Sender>>,
    logger: Addr<Logger>,
    writer: Addr<Writer>,
    answers: HashMap<String, HashMap<String, Answer>>,
    transactions: HashMap<String, Transaction>,
}

impl TransactionManager {
    pub fn new(logger: Addr<Logger>, writer: Addr<Writer>) -> Self {
        TransactionManager {
            bank: None,
            airline: None,
            hotel: None,
            logger,
            writer,
            answers: HashMap::<String, HashMap<String, Answer>>::new(),
            transactions: HashMap::<String, Transaction>::new(),
        }
    }
}

impl Actor for TransactionManager {
    type Context = SyncContext<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendAddr(pub Option<Addr<Sender>>, pub Option<Addr<Sender>>, pub Option<Addr<Sender>>);

impl Handler<SendAddr> for TransactionManager {
    type Result = ();

    fn handle(&mut self, msg: SendAddr, ctx: &mut SyncContext<Self>) -> Self::Result {
        self.airline = msg.0;
        self.hotel = msg.1;
        self.bank = msg.2;
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTransactionToEntities(pub Transaction);

impl Handler<SendTransactionToEntities> for TransactionManager {
    type Result = ();

    fn handle(&mut self, msg: SendTransactionToEntities, ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction = msg.0;

        self.answers.insert(transaction.id.clone(), HashMap::from([
            ("airline".to_string(), Answer::Pending),
            ("bank".to_string(), Answer::Pending),
            ("hotel".to_string(), Answer::Pending),
        ]));

        self.transactions.insert(transaction.id.clone(), transaction.clone());
        match &self.airline {
            Some(addr) => {
                addr.do_send(SendTransaction(transaction.clone()));
            }
            None => {
                ctx.address().do_send(ProcessEntityAnswer(EntityAnswer {
                    entity_name: "airline".to_string(),
                    answer: Answer::Failed,
                    transaction_id: transaction.id.clone(),
                }));
                self.logger.do_send(Log(format!("No se pudo enviar la transaccion n°:{} a  aerolinea!", transaction.id.clone())));
            }
        }

        match &self.bank {
            Some(addr) => {
                addr.do_send(SendTransaction(transaction.clone()));
            }
            None => {
                ctx.address().do_send(ProcessEntityAnswer(EntityAnswer {
                    entity_name: "bank".to_string(),
                    answer: Answer::Failed,
                    transaction_id: transaction.id.clone(),
                }));
                self.logger.do_send(Log(format!("No se pudo enviar la transaccion n°:{} a banco!", transaction.id.clone())));
            }
        }
        match &self.hotel {
            Some(addr) => {
                addr.do_send(SendTransaction(transaction.clone()));
            }
            None => {
                ctx.address().do_send(ProcessEntityAnswer(EntityAnswer {
                    entity_name: "hotel".to_string(),
                    answer: Answer::Failed,
                    transaction_id: transaction.id.clone(),
                }));
                self.logger.do_send(Log(format!("No se pudo enviar la transaccion n°: {} a hotel!", transaction.id.clone())));
            }
        }

        self.logger.do_send(Log(
            format!("Se envía paquete de id {} a entidades para procesamiento", transaction.id)
        ));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessEntityAnswer(pub EntityAnswer);

impl Handler<ProcessEntityAnswer> for TransactionManager {

    type Result = ();

    fn handle(&mut self, msg: ProcessEntityAnswer, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let entity_answer = msg.0;

        if let Some(transaction_answers) =
        self.answers.get_mut(&entity_answer.transaction_id)
        {
            transaction_answers.insert(entity_answer.entity_name.clone(),
                                       entity_answer.answer.clone());

            self.logger.do_send(Log(
                format!("Se registra respuesta de entidad {} para transacción de id {} con resultado: {:?}",
                        entity_answer.entity_name, entity_answer.transaction_id, entity_answer.answer)
            ));

            let completed: bool = transaction_answers
                .iter()
                .all(|(_n, state)| *state != Answer::Pending);
            println!("Respuestas: {:?} - ¿Completo? {}", transaction_answers, completed);

            if completed {
                let ok_entities = transaction_answers
                    .iter()
                    .filter(|(_n, state)| *state == &Answer::Ok);

                let mut transaction_result = TransactionResult {
                    transaction_id: entity_answer.transaction_id.clone(),
                    success: true,
                };
                let mut message_type = "commit";

                let has_fails: bool = transaction_answers
                    .iter()
                    .any(|(_n, state)| *state == Answer::Failed);

                if has_fails {
                    transaction_result.success = false;
                    message_type = "rollback";
                    if let Some(transaction) =
                    self.transactions.get(entity_answer.transaction_id.as_str())
                    {
                        self.writer.do_send(WriteTransaction(transaction.clone()));
                        self.logger.do_send(Log(format!(
                            "Se escribe transacción de id {} en archivo de fallas",
                            transaction.id
                        )));
                    }
                }

                ok_entities.for_each(|(entity_name, answer)| {
                    match entity_name.as_str() {
                        "airline" => {
                            match &self.airline {
                                Some(x) => {
                                    x.do_send(SendConfirmationOrRollback(transaction_result.clone()));
                                }
                                None => {
                                    //todo
                                }
                            }
                        }
                        "bank" => {
                            match &self.bank {
                                Some(x) => {
                                    x.do_send(SendConfirmationOrRollback(transaction_result.clone()));
                                }
                                None => {
                                    //todo
                                }
                            }
                        }
                        "hotel" => {
                            match &self.hotel {
                                Some(x) => {
                                    x.do_send(SendConfirmationOrRollback(transaction_result.clone()));
                                }
                                None => {
                                    //todo
                                }
                            }
                        }
                        _ => {}
                    };
                    self.logger.do_send(Log(format!("Se envía mensage de {} para la transaccion \
                    {} a entidad {} para procesamiento", message_type, entity_answer.transaction_id, entity_name)));
                })
            }
        }
    }
}
