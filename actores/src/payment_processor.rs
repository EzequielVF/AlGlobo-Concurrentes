use std::collections::HashMap;
use std::sync::mpsc::Sender;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use crate::external_entity::{EE_NewPayment, ExternalEntity};

#[derive(Clone)]
pub struct PaqueteTuristico {
    pub id: usize,
    pub precio: usize,
}

type TransactionState = HashMap<String, RequestState>;

pub struct PaymentProcessor {
    bank_address: Addr<ExternalEntity>,
    airline_address: Addr<ExternalEntity>,
    hotel_address: Addr<ExternalEntity>,
    entity_answers: HashMap<usize, TransactionState>,
    logger_tx: Addr<Logger>
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestState {
    Sent,
    Ok,
    Failed,
}

impl PaymentProcessor {
    pub fn new(bank_address: Addr<ExternalEntity>,
               airline_address: Addr<ExternalEntity>,
               hotel_address: Addr<ExternalEntity>,
               logger: Addr<Logger>,
    ) -> Self {
        PaymentProcessor {
            bank_address,
            airline_address,
            hotel_address,
            entity_answers: HashMap::<usize, HashMap<String, RequestState>>::new(),
            logger_tx: logger
        }
    }
}

impl Actor for PaymentProcessor {
    type Context = Context<Self>;
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct PP_NewPayment(pub PaqueteTuristico);

impl Handler<PP_NewPayment> for PaymentProcessor {
    type Result = ();

    fn handle(&mut self, msg: PP_NewPayment, ctx: &mut Context<Self>) -> Self::Result {
        self.bank_address.try_send(EE_NewPayment((msg.0).clone(), ctx.address().clone()));
        self.airline_address.try_send(EE_NewPayment((msg.0).clone(), ctx.address().clone()));
        self.hotel_address.try_send(EE_NewPayment((msg.0).clone(), ctx.address().clone()));

        self.entity_answers.insert(msg.0.id, HashMap::from([
            ("BANK".to_string(), RequestState::Sent),
            ("AIRLINE".to_string(), RequestState::Sent),
            ("HOTEL".to_string(), RequestState::Sent)
        ]));
    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct EntityAnswer(pub String, pub usize, pub RequestState);

impl Handler<EntityAnswer> for PaymentProcessor {
    type Result = ();

    fn handle(&mut self, msg: EntityAnswer, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(transaction) = self.entity_answers.get_mut(&msg.1) {

            transaction.insert(msg.0.clone(), msg.2.clone());
            &self.logger_tx.try_send(Log(format!("[{}-ID:{}-Estado:{:?}]", msg.0, msg.1, msg.2)));
            if transaction.iter().all(|(_name, state)| *state != RequestState::Sent) {
                println!("[EntityAnswer] Recibí 3 respuestas");
                transaction.iter()
                    .for_each(|(n, s)| println!("[Hash] Name: {} - State: {:?}", n, s));
            }
        }
    }
}

