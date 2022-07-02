use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use actix::{Actor, Addr, Context, Handler, SyncContext};
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

        let filename = format!("{}.log", name);
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
pub struct WriteErrorTransaction(pub Transaction);

impl Handler<WriteErrorTransaction> for Writer {
    type Result = ();

    fn handle(&mut self, msg: WriteErrorTransaction, _ctx: &mut Context<Self>) -> Self::Result {
        let transaction = msg.0;
        let message = format!("{},{}\n", transaction.id, transaction.precio);
        self.file
            .write_all(message.as_bytes())
            .expect("Error escribiendo archivo de log");
    }
}