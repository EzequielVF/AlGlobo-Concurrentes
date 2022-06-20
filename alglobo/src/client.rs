use std::fs::File;
use std::io::{BufRead, BufReader};
use actix::Actor;
use actix_rt::System;
use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull, Unknown};
use crate::banco::Prueba;
use crate::banco::Procesar;

const FILE: &str = "alglobo/src/archivo.csv";

pub struct PaqueteTuristico {
    pub id: u32,
    pub precio: u32,
    pub vuelo: String,
    pub hotel: String,
}

pub fn run(ip:&str) {

    let system = System::new();
    system.block_on(async {
        let banco_addr = crate::banco::Banco::new("127.0.0.1", "3001", String::from(FILE)).start();
        let resp = banco_addr.send(Prueba()).await;

        let paquetes_turisticos = parsear_paquetes(FILE);
        for paquete in paquetes_turisticos {
            let resp = banco_addr.send(Procesar(paquete)).await;
        }

        System::current().stop();
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

