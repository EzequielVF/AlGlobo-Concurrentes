use actix::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use actix::{Actor, Addr, Handler, SyncContext, Message, Context};
use crate::Logger;
use crate::reader::abrir_archivo_paquetes;
use crate::types::Transaction;

pub struct Writer {
    file: File,
    logger_address: Addr<Logger>,
}

impl Actor for Writer {
    type Context = SyncContext<Self>;
}

impl Writer {
    pub fn new(path: &str, addr: Addr<Logger>) -> Self {

        let filename = format!("fails.csv");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .expect("Error creando archivo de log");
        Writer {
            file: file,
            logger_address: addr
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
        self.file
            .write_all(message.as_bytes())
            .expect("Error escribiendo archivo de log");
    }
}