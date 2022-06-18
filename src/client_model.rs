use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::comunicacion::Message;
use crate::comunicacion::Message::{Error, Pay, Succesfull};

fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);

    buffer.extend_from_slice(data.as_bytes());
}

#[allow(dead_code)]
struct PaqueteTuristico {
    id: u32,
    precio: u32,
    vuelo: String,
    hotel: String,
}

/// Módulo de Procesamiento de Pagos
///
/// A partir un archivo .csv
/// con formato "id,precio-paquete,aeropuerto-origen-aeropuerto-destino,nombre-hotel"
///
/// El procesamiento de cada pago se ejecuta concurrentemente ¿fork-join?
pub fn run(ip:&str, port:&str) {
    // Conectar con aerolinea
    let mut airline_channel = conectar_con_servidor("127.0.0.1", "3000", String::from("Airline"));
    // Conectar con banco
    let bank_channel = conectar_con_servidor("127.0.0.1", "3001", String::from("Bank"));
    // Conectar con hotel
    let hotel_channel = conectar_con_servidor("127.0.0.1", "3002", String::from("Hotel"));

    // let argumentos: Vec<String> = env::args().collect();
    // let ruta = &argumentos[1];

    // let paquetes_turisticos = parsear_paquetes(ruta);

    //
    // for paquetes_turistico in paquetes_turisticos {
    //     enviar_pago_a_banco(&mut stream, paquetes_turistico);
    //     leer_respuesta(&mut stream);
    // }

    match bank_channel {
        Ok(bank) => {
            let mut channel = bank;
            loop {
                enviar_pago_a_banco(&mut channel);
            }
        }
        Err(_) => {

        }
    }



}
fn conectar_con_servidor(ip:&str, port:&str, tipo: String) -> Result<TcpStream, std::io::Error>{
    let address= format!("{}:{}", ip, port);
    println!("<CLIENTE> Intentando establecer conexión con:  {}",address);
    let mut stream = TcpStream::connect( address)?;
    Ok(stream)
}

//fn enviar_pago_a_banco(stream: &mut TcpStream, paquete: PaqueteTuristico)
fn enviar_pago_a_banco(stream: &mut TcpStream) {
    //let cantidad_pago = paquete.precio.to_string();
    let cantidad_pago = String::from("500");
    let size = (cantidad_pago.len() + 1) as u8;
    let buffer = [Pay.into(), size]; // Me armo el buffer de aviso, primer byte tipo, segundo byte tamaño
    match stream.write_all(&buffer) {
        /*Ok(_) => {
            println!(
                "<Cliente> Mensaje (id: {}) enviado correctamente!",
                paquete.id
            );
        }*/ //Hardcodeado mientras que no haya archivo
        Ok(_) => {
            println!(
                "<Cliente> Mensaje (id: 0) enviado correctamente!",
            );
        }
        Err(_) => {
            println!("<Cliente> No me pude contactar con el banco!");
            // exit(0);
        }
    }

    let mut buffer_envio: Vec<u8> = Vec::with_capacity(size.into()); // Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut buffer_envio, cantidad_pago);
    match stream.write(&buffer_envio) {
        /*Ok(_) => {
            println!(
                "<Cliente> Mensaje (id: {}) enviado correctamente!",
                paquete.id
            );
        }*/
        Ok(_) => {
            println!(
                "<Cliente> Mensaje (id: 0) enviado correctamente!",
            );
        }
        Err(_) => {
            println!("<Cliente> No me pude contactar con el banco!");
        }
    }
}

pub fn leer_respuesta(stream: &mut TcpStream) {
    let mut num_buffer = [0u8; 2];
    let _aux = stream.read_exact(&mut num_buffer);
    match Message::from(num_buffer[0]) {
        Succesfull => {
            println!("<Cliente> Pago procesado correctamente!\n");
        }
        Error => {
            println!("<Cliente> No pudo procesarse el pago!\n");
        }
        _ => {
            println!("<Cliente> No sé qué me contestó el server!\n");
        }
    }
}

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