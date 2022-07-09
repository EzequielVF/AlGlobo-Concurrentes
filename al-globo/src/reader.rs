use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, SyncContext};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};

use crate::logger::Log;
use crate::types::Transaction;
use crate::{Logger, TransactionManager};
use crate::transaction_manager::SendTransactionToEntities;

pub struct Reader {
    file: BufReader<File>,
    transaction_manager: Addr<TransactionManager>,
    logger: Addr<Logger>,
}

impl Reader {
    pub fn new(path: &str, transaction_manager: Addr<TransactionManager>, logger: Addr<Logger>) -> Self {
        let file = open_file(path);
        match file {
            Ok(f) => Reader {
                file: f,
                transaction_manager,
                logger,
            },
            Err(_) => {
                let file = OpenOptions::new()
                    .write(false)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .expect("Error Leyendo archivo");
                let reader = BufReader::new(file);
                Reader {
                    file: reader,
                    transaction_manager,
                    logger,
                }
            }
        }
    }
}


impl Actor for Reader {
    type Context = SyncContext<Self>;
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ReadTransaction();

impl Handler<ReadTransaction> for Reader {
    type Result = ();

    fn handle(&mut self,_msg: ReadTransaction,ctx: &mut <Reader as Actor>::Context) -> Self::Result {
        
        let mut buffer = String::from("");

        if let Ok(line) = self.file.read_line(&mut buffer) {
            if line > 0 {
                let splitted_line: Vec<&str> = buffer.split(',').collect();
                let transaction = Transaction {
                    id: splitted_line[0].to_string(),
                    precio: splitted_line[1].to_string(),
                };
                self.logger.do_send(Log(format!("Se leyó paquete con id {} - y precio: {}",
                    splitted_line[0], splitted_line[1])));
                self.transaction_manager.do_send(SendTransactionToEntities(transaction));
                ctx.address().do_send(ReadTransaction());
            }
        }
    }
}

pub fn open_file(ruta: &str) -> Result<BufReader<File>, std::io::Error> {
    match File::open(ruta) {
        Ok(file) => Ok(BufReader::new(file)),
        Err(err) => Err(err),
    }
}