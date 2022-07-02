use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};

use crate::{Logger, TransactionManager};
use crate::logger::Log;
use crate::transaction_manager::SendTransactionToEntities;
use crate::types::Transaction;

/// Parser para archivo de paquetes turísticos
pub struct Reader {
    buffer: BufReader<File>,
    pp_address: Addr<TransactionManager>,
    logger_address: Addr<Logger>,
}

impl Reader {
    pub fn new(path: &str, addr: Addr<TransactionManager>, addr_log: Addr<Logger>) -> Self {
        let t = abrir_archivo_paquetes(path);
        match t {
            Ok(b) => Reader {
                buffer: b,
                pp_address: addr,
                logger_address: addr_log,
            },
            Err(_) => {
                println!("<MAIN> No pude abrir el archivo!");
                let file = OpenOptions::new()
                    .write(false)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .expect("Error creando archivo de log");

                let reader = BufReader::new(file);
                Reader {
                    buffer: reader,
                    pp_address: addr,
                    logger_address: addr_log,
                }
            }
        }
    }
}

pub fn abrir_archivo_paquetes(ruta: &str) -> Result<BufReader<File>, std::io::Error> {
    match File::open(ruta) {
        Ok(archivo_pagos) => Ok(BufReader::new(archivo_pagos)),
        Err(err) => Err(err),
    }
}

impl Actor for Reader {
    type Context = Context<Self>;
}

/// Mensaje para solicitar al parser un nuevo paquete turístico
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ParseTouristPackage();

impl Handler<ParseTouristPackage> for Reader {
    type Result = ();

    fn handle(&mut self, _msg: ParseTouristPackage, ctx: &mut Context<Self>) -> Self::Result {
        let mut buffer = String::from("");

        if let Ok(_line) = self.buffer.read_line(&mut buffer) {
            let splitted_package_line: Vec<&str> = buffer.split(',').collect();

            let transaction = Transaction {
                id: splitted_package_line[0].to_string(),
                precio: splitted_package_line[1].to_string(),
            };
            self.logger_address.do_send(Log(format!(
                "Se leyó paquete con id {} - y precio: {}",
                splitted_package_line[0], splitted_package_line[1]
            )));

            self.pp_address.do_send(SendTransactionToEntities(transaction));

            ctx.address().do_send(ParseTouristPackage());

            // thread::sleep(Duration::from_secs(1));
        }
    }
}
