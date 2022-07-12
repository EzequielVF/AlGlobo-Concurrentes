use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

use actix::{Actor, Addr, Handler, Message, SyncContext};

use crate::{Logger, Stats, TransactionManager};
use crate::logger::Log;
use crate::stats::StartTime;
use crate::transaction_manager::SendTransactionToEntities;
use crate::types::Transaction;

pub struct Reader {
    file: BufReader<File>,
    transaction_manager: Addr<TransactionManager>,
    logger: Addr<Logger>,
    keep_running: bool,
    stats: Addr<Stats>
}

impl Reader {
    pub fn new(path: &str, transaction_manager: Addr<TransactionManager>, logger: Addr<Logger>, stats: Addr<Stats>) -> Self {
        let file = open_file(path);
        let keep_running = true;

        match file {
            Ok(f) => Reader {
                file: f,
                transaction_manager,
                logger,
                keep_running,
                stats
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
                    keep_running,
                    stats
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

    fn handle(&mut self, _msg: ReadTransaction, ctx: &mut <Reader as Actor>::Context) -> Self::Result {
        let mut buffer = String::from("");

        if let Ok(line) = self.file.read_line(&mut buffer) {
            if line > 0 {
                let split_line: Vec<&str> = buffer.split(',').collect();
                let transaction = Transaction {
                    id: split_line[0].to_string(),
                    precio: split_line[1].to_string(),
                };
                self.logger.do_send(Log("READER".to_string(), format!("Se leyó paquete con id {} - y precio: {}",
                                                                      split_line[0], split_line[1])));
                self.transaction_manager.do_send(SendTransactionToEntities(transaction.clone()));
                self.stats.do_send(StartTime(transaction.id.clone()));
                thread::sleep(Duration::from_secs(3));

                self.logger.do_send(Log("[READER]".to_string(), format!("keep running: {}", self.keep_running)));
                if self.keep_running {
                    ctx.address().do_send(ReadTransaction());
                }
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopReading();

impl Handler<StopReading> for Reader {
    type Result = ();

    fn handle(&mut self, _msg: StopReading, _ctx: &mut <Reader as Actor>::Context) -> Self::Result {
        self.logger.do_send(Log("[READER]".to_string(), format!("~~~~~~~~~ Oh no! Me están stoppeando!")));
        self.keep_running = false;
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartReading();

impl Handler<StartReading> for Reader {
    type Result = ();

    fn handle(&mut self, _msg: StartReading, ctx: &mut <Reader as Actor>::Context) -> Self::Result {
        self.logger.do_send(Log("[READER]".to_string(), format!("~~~~~~~~~ Leyendo de nuevooooooooooo!11111!!")));
        self.keep_running = true;
        ctx.address().do_send(ReadTransaction());
    }
}

pub fn open_file(ruta: &str) -> Result<BufReader<File>, std::io::Error> {
    match File::open(ruta) {
        Ok(file) => Ok(BufReader::new(file)),
        Err(err) => Err(err),
    }
}