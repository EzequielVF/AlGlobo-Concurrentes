use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use crate::{Log, Logger, PaqueteTuristico, PaymentProcessor, PP_NewPayment};

pub struct Reader {
    buffer: BufReader<File>,
    pp_address: Addr<PaymentProcessor>,
    logger_address: Addr<Logger>,
}

impl Reader {
    pub fn new(path: &str, addr: Addr<PaymentProcessor>, addr_log: Addr<Logger>) -> Self {
        let t = abrir_archivo_paquetes(path);
        match t {
            Ok(b) => {
                return Reader {
                    buffer: b,
                    pp_address: addr,
                    logger_address: addr_log,
                };
            }
            Err(_) => {
                println!("<MAIN> No pude abrir el archivo!");
                let file = OpenOptions::new()
                    .write(false)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .expect("Error creando archivo de log");

                let mut reader = BufReader::new(file);
                return Reader {
                    buffer: reader,
                    pp_address: addr,
                    logger_address: addr_log
                };
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

    fn handle(&mut self, msg: LeerPaquete, ctx: &mut Context<Self>) -> Self::Result {
        let mut buffer = String::from("");
        let mut aux = self.buffer.read_line(&mut buffer);

        if aux.unwrap() > 0 {
            let paquete: Vec<&str> = buffer.split(',').collect();
            let paquete_aux = PaqueteTuristico {
                id: paquete[0].parse::<usize>().unwrap(),
                precio: paquete[1].parse::<usize>().unwrap(),
            };
            self.logger_address.try_send(Log(format!("Se leyó paquete con id {}- y precio:{}",paquete[0], paquete[1])));
            self.pp_address.try_send(PP_NewPayment(paquete_aux));
            ctx.address().try_send(LeerPaquete());
        }
    }
}
