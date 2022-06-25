use std::fs::File;
use std::io::{BufRead, BufReader};

use actix::Actor;
use actix_rt::{Arbiter, System};

use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull, Unknown};
use crate::entity_actor::ConnectionStatus;
use crate::procesador_pagos::ProcesadorPagos;
use crate::procesador_pagos::Procesar;

const IP: &str = "127.0.0.1";
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
    let mut vec = vec!();
    let paquetes_turisticos = parsear_paquetes(FILE); //ver de hacer un lector async o convertirlo en actor
    system.block_on(async {
        let addr_pp = ProcesadorPagos::new(IP).start();
        for paquete in paquetes_turisticos {
            vec.push(addr_pp.send(Procesar(paquete)));
        }
    });

    system.block_on(async {
        for x in vec {
            x.await; // esperamos la respuesta del PP
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

