use std::fs::{File, OpenOptions};
use std::io::{Write};

use actix::{Actor, Addr, Handler, Message, SyncContext};

use crate::types::Transaction;
use crate::Logger;
use crate::logger::Log;

pub struct Writer {
    file: File,
    logger: Addr<Logger>,
}

impl Actor for Writer {
    type Context = SyncContext<Self>;
}

impl Writer {
    pub fn new(path: &str, addr: Addr<Logger>) -> Self {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect("Error creando archivo de fallas");

        Writer {
            file,
            logger: addr,
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WriteTransaction(pub Transaction);

impl Handler<WriteTransaction> for Writer {
    type Result = ();

    fn handle(&mut self, msg: WriteTransaction, _ctx: &mut  SyncContext<Self>) -> Self::Result {
        let transaction = msg.0;
        let message = format!("{},{}\n", transaction.id, transaction.precio);
        self.file.write_all(message.as_bytes());
    }
}
