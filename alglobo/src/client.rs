use std::fs::File;
use std::io::{BufRead, BufReader};
use actix::Actor;
use actix_rt::System;
use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull, Unknown};
use crate::entity_actor::ConnectionStatus;
use crate::entity_actor::ProcesarPaquete;

const IP: &str = "127.0.0.1";
const PORT_AEROLINEA: &str = "3000";
const PORT_BANCO: &str = "3001";
const PORT_HOTEL: &str = "3002";
const FILE: &str = "alglobo/src/archivo.csv";


#[derive(Clone)]
pub struct PaqueteTuristico {
    pub id: u32,
    pub precio: u32,
    pub vuelo: String,
    pub hotel: String,
}

pub fn run() {

    let system = System::new();
    system.block_on(async {
        let banco_addr = crate::entity_actor::EntityActor::new(IP, PORT_BANCO, String::from(FILE)).start();
        let aerolinea_addr = crate::entity_actor::EntityActor::new(IP, PORT_AEROLINEA, String::from(FILE)).start();
        let hotel_addr = crate::entity_actor::EntityActor::new(IP, PORT_HOTEL, String::from(FILE)).start();

        let paquetes_turisticos = parsear_paquetes(FILE);
        for paquete in paquetes_turisticos {
            let resp = banco_addr.send(ProcesarPaquete(paquete.clone())).await;
            let resp = aerolinea_addr.send(ProcesarPaquete(paquete.clone())).await;
            let resp = hotel_addr.send(ProcesarPaquete(paquete.clone())).await;
        }

        //System::current().stop();
    });
    system.run();
}
//Funciones auxiliareas despues capaz las movemos a otro lado!
fn parsear_paquetes(ruta: &str) -> Vec<PaqueteTuristico> {
    let mut paquetes_turisticos: Vec<PaqueteTuristico> = Vec::new();

    match abrir_archivo_paquetes(ruta) {
        Ok(archivo) => {
            for read_result in archivo.lines() {
                match read_result {
                    Ok(line) => {
                        let paquete: Vec<&str> = line.split(',').collect();

                        paquetes_turisticos.push(PaqueteTuristico {
                            id: paquete[0].parse::<u32>().unwrap(),
                            precio: paquete[1].parse::<u32>().unwrap(),
                            vuelo: String::from(paquete[2]),
                            hotel: String::from(paquete[3]),
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

