use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

use actix::Actor;
use actix_rt::{Arbiter, System};

use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull, Unknown};
use crate::external_entity::ExternalEntity;
use crate::payment_processor::{PaqueteTuristico, PaymentProcessor, PP_NewPayment};
use crate::reader::{LeerPaquete, Reader};


mod external_entity;
mod payment_processor;
mod comunicacion;
mod reader;

const PORT_AEROLINEA: &str = "3000";
const PORT_BANCO: &str = "3001";
const PORT_HOTEL: &str = "3002";
const IP: &str = "127.0.0.1";

const FILE: &str = "alglobo/src/archivo.csv";

fn parsear_paquetes(ruta: &str) -> Vec<PaqueteTuristico> {
    let mut paquetes_turisticos: Vec<PaqueteTuristico> = Vec::new();

    match abrir_archivo_paquetes(ruta) {
        Ok(archivo) => {
            for read_result in archivo.lines() {
                match read_result {
                    Ok(line) => {
                        let paquete: Vec<&str> = line.split(',').collect();

                        paquetes_turisticos.push(PaqueteTuristico {
                            id: paquete[0].parse::<usize>().unwrap(),
                            precio: paquete[1].parse::<usize>().unwrap(),
                        })
                    }
                    Err(e) => {
                        eprintln!("Ups... {}", e)
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    paquetes_turisticos
}

fn abrir_archivo_paquetes(ruta: &str) -> Result<BufReader<File>, std::io::Error> {
    match File::open(ruta) {
        Ok(archivo_pagos) => Ok(BufReader::new(archivo_pagos)),
        Err(err) => Err(err),
    }
}


#[actix_rt::main]
async fn main() {
    let (logger_send, logger_receive) = mpsc::channel();

    let paquetes_turisticos = parsear_paquetes(FILE);

    let execution = async {
        let bank_address = ExternalEntity::new("BANK", IP, PORT_BANCO).start();
        let airline_address = ExternalEntity::new("AIRLINE", IP, PORT_AEROLINEA).start();
        let hotel_address = ExternalEntity::new("HOTEL", IP, PORT_HOTEL).start();

        let pp_addr = PaymentProcessor::new(bank_address, airline_address, hotel_address, logger_send).start();

        for paquete in paquetes_turisticos {
            pp_addr.try_send(PP_NewPayment(paquete));
        }
    };

    thread::spawn(move || {
        // let logger = Logger::new("payment_processor");
        loop {
            if let Ok(rx) = logger_receive.recv() {
                println!("[LOGGER] {}", rx)
            }
        }
    });

    let arbitro = Arbiter::new();
    arbitro.spawn(execution);
    arbitro.join();
}

