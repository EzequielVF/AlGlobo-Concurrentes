use std::net::TcpStream;
use std::sync::{Arc, Mutex};
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
            name: name.to_string(),
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
        let mut channel = self.stream.try_clone().unwrap();
        self.logger.do_send(Log(
            format!("Se envía transacción de id {} al servidor  [{}]",transaction.id, self.name)));
            send_package(&mut channel, transaction.clone(), &self.name);
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

        let mut channel = self.stream.try_clone().unwrap();
        send_transaction_result(&mut channel, transaction_result.clone());
        self.logger.do_send(Log(format!("Se envía mensage {} para transacción {} al servidor [{}]",
                    message, transaction_result.transaction_id, self.name)));
            }
        }
