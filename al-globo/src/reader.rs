use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};

use crate::{Log, Logger, PaqueteTuristico, PayProcNewPayment, PaymentProcessor};

pub struct Reader {
    buffer: BufReader<File>,
    pp_address: Addr<PaymentProcessor>,
    logger_address: Addr<Logger>,
}

impl Reader {
    pub fn new(path: &str, addr: Addr<PaymentProcessor>, addr_log: Addr<Logger>) -> Self {
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

fn abrir_archivo_paquetes(ruta: &str) -> Result<BufReader<File>, std::io::Error> {
    match File::open(ruta) {
        Ok(archivo_pagos) => Ok(BufReader::new(archivo_pagos)),
        Err(err) => Err(err),
    }
}

impl Actor for Reader {
    type Context = Context<Self>;
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct LeerPaquete();

impl Handler<LeerPaquete> for Reader {
    type Result = ();

    fn handle(&mut self, _msg: LeerPaquete, ctx: &mut Context<Self>) -> Self::Result {
        let mut buffer = String::from("");
        let aux = self.buffer.read_line(&mut buffer);

        if aux.unwrap() > 0 {
            let paquete: Vec<&str> = buffer.split(',').collect();
            let paquete_aux = PaqueteTuristico {
                id: paquete[0].parse::<usize>().unwrap(),
                precio: paquete[1].parse::<usize>().unwrap(),
            };
            self.logger_address.do_send(Log(format!(
                "Se leyó paquete con id {}- y precio:{}",
                paquete[0], paquete[1]
            )));
            self.pp_address.do_send(PayProcNewPayment(paquete_aux));

            ctx.address().do_send(LeerPaquete());

            thread::sleep(Duration::from_secs(1));
        }
    }
}
