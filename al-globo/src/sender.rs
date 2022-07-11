use std::net::TcpStream;
use actix::{Actor, Addr, Handler, Message, SyncContext};

use crate::{Logger};
use crate::communication::{send_package, send_transaction_result};
use crate::logger::Log;
use crate::types::{Transaction, TransactionResult};

pub struct Sender {
    name: String,
    stream: TcpStream,
    logger: Addr<Logger>,
}

impl Actor for Sender {
    type Context = SyncContext<Self>;
}

impl Sender {
    pub fn new(name: String,socket: TcpStream,logger: Addr<Logger>) -> Self {
        Sender {
            name,
            stream : socket,
            logger}
    }
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTransaction(pub Transaction);

impl Handler<SendTransaction> for Sender {
    type Result = ();

    fn handle(&mut self,msg: SendTransaction, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction = msg.0;
       /* let mut channel = self.stream.try_clone().expect("No pude clonar el stream para pasarlo!");
        self.logger.do_send(Log("SENDER".to_string(),
            format!("Se envía transacción de id {} al servidor  [{}]",transaction.id, self.name)));
            send_package(&mut channel, transaction.clone(), &self.name);*/
        match &mut self.stream.try_clone() {
            Ok(channel) => {
                self.logger.do_send(Log("SENDER".to_string(), format!("Se envía transacción de id {} al servidor  [{}]",transaction.id, self.name)));
                send_package(channel, transaction, &self.name);
            }
            Err(_) => {
                self.logger.do_send(Log("SENDER".to_string(), "No pude clonar el socket!".to_string()));
            }
        }
    }
}


#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct SendConfirmationOrRollback(pub TransactionResult);

impl Handler<SendConfirmationOrRollback> for Sender {
    type Result = ();

    fn handle(
        &mut self, msg: SendConfirmationOrRollback, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction_result = msg.0;

        let mut message = "confirmacion";

        if !transaction_result.success {
            message = "rollback";
        }

        //let mut channel = self.stream.try_clone().expect("No pude clonar el stream para pasarlo!");

        match &mut self.stream.try_clone() {
            Ok(channel) => {
                send_transaction_result(channel, transaction_result.clone());
                self.logger.do_send(Log("SENDER".to_string(), format!("Se envía mensaje {} para transacción {} al servidor [{}]",
                                                message, transaction_result.transaction_id, self.name)));
            }
            Err(_) => {
                self.logger.do_send(Log("SENDER".to_string(), "No pude clonar el socket!".to_string()));
            }
        }
        /*send_transaction_result(&mut channel, transaction_result.clone());
        self.logger.do_send(Log("SENDER".to_string(), format!("Se envía mensaje {} para transacción {} al servidor [{}]", message, transaction_result.transaction_id, self.name))); }
        */
    }
}

