use std::collections::HashMap;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};

use crate::external_entity::{
    ExtEntNewPayment, ExternalEntity, TransactionResult, TransactionResultMessage,
};
use crate::{Log, Logger};

#[derive(Clone)]
pub struct TouristPackage {
    pub id: usize,
    pub precio: usize,
}

type TransactionState = HashMap<String, RequestState>;

pub struct PaymentProcessor {
    bank_address: Addr<ExternalEntity>,
    airline_address: Addr<ExternalEntity>,
    hotel_address: Addr<ExternalEntity>,
    entity_answers: HashMap<usize, TransactionState>,
    logger_address: Addr<Logger>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestState {
    Sent,
    Ok,
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
                println!("[EntityAnswer] Recibí 3 respuestas");
                transaction
                    .iter()
                    .for_each(|(n, s)| println!("[Hash] Name: {} - State: {:?}", n, s));

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
