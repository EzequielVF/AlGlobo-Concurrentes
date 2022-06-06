use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Módulo de Procesamiento de Pagos
///
/// A partir un archivo .csv
/// con formato "id,precio-paquete,aeropuerto-origen-aeropuerto-destino,nombre-hotel"
///
/// El procesamiento de cada pago se ejecuta concurrentemente ¿fork-join?
fn main() {
    let argumentos: Vec<String> = env::args().collect();
    let ruta = &argumentos[1];

    match File::open(ruta) {
        Ok(archivo_pagos) => {
            let reader = BufReader::new(archivo_pagos);

            for read_result in reader.lines() {
                match read_result {
                    Ok(line) => { println!("{}", line); }
                    Err(_) => { println!("Ups...") }
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err)
        }
    }
}
