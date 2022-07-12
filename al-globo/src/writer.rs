use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};

use actix::{Actor, Addr, Handler, Message, SyncContext};

use crate::types::{Transaction};
use crate::{Answer, Logger};

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
            .open(path)
            .expect("Error creando archivo de escritura");

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

    fn handle(&mut self, msg: WriteTransaction, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction = msg.0;
        let message = format!("{},{}\n", transaction.id, transaction.precio);
        self.file.write_all(message.as_bytes());
        self.file.flush();
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WriteAnswers(pub HashMap<String, HashMap<String, Answer>>);

impl Handler<WriteAnswers> for Writer {
    type Result = ();

    fn handle(&mut self, msg: WriteAnswers, _ctx: &mut SyncContext<Self>) -> Self::Result {
        self.file.set_len(0);
        self.file.seek(SeekFrom::Start(0));

        let answers = msg.0;

        // TODO: sacar este unwrap
        let serialized = serde_json::to_string(&answers).unwrap();

        self.file.write_all(serialized.as_bytes());
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WriteTransactions(pub HashMap<String, Transaction>);

impl Handler<WriteTransactions> for Writer {
    type Result = ();

    fn handle(&mut self, msg: WriteTransactions, _ctx: &mut SyncContext<Self>) -> Self::Result {
        self.file.set_len(0);
        self.file.seek(SeekFrom::Start(0));

        let transactions = msg.0;

        // TODO: sacar este unwrap
        let serialized = serde_json::to_string(&transactions).unwrap();

        self.file.write_all(serialized.as_bytes());
    }
}
