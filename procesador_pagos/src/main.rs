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

    match abrir_archivo_paquetes(ruta) {
        Ok(archivo) => {
            for read_result in archivo.lines() {
                match read_result {
                    Ok(line) => { println!("{}", line); }
                    Err(e) => { eprintln!("Ups... {}", e) }
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

fn abrir_archivo_paquetes(ruta: &str) -> Result<BufReader<File>, std::io::Error> {
    match File::open(ruta) {
        Ok(archivo_pagos) => {
            Ok(BufReader::new(archivo_pagos))
        }
        Err(err) => {
            Err(err)
        }
    }
}
