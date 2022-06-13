use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::TcpStream;

fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);
    let data_bytes = data.as_bytes();
    for i in 0..data_bytes.len() {
        buffer.push(data_bytes[i]);
    }
}

#[allow(dead_code)]
struct PaqueteTuristico {
    id: u32,
    precio: u32,
    vuelo: String,
    hotel: String,
}

pub enum Message {
    Pay,
    Succesfull,
    Error,
    Unknown
}
// Fijarse si se puede usar un crate!
impl From<u8> for Message {
    fn from(code: u8) -> Message {
        match code & 0xF0 {
            0x00 => Message::Pay,
            0x10 => Message::Succesfull,
            0x20 => Message::Error,
            _ => Message::Unknown
        }
    }
}

impl From<Message> for u8 {
    fn from(code: Message) -> u8 {
        match code {
            Message::Pay => 0x00,
            Message::Succesfull => 0x10,
            Message::Error => 0x20,
            _ => 0x99
        }
    }
}

/// Módulo de Procesamiento de Pagos
///
/// A partir un archivo .csv
/// con formato "id,precio-paquete,aeropuerto-origen-aeropuerto-destino,nombre-hotel"
///
/// El procesamiento de cada pago se ejecuta concurrentemente ¿fork-join?
fn main() {
    let argumentos: Vec<String> = env::args().collect();
    let ruta = &argumentos[1];

    let paquetes_turisticos = parsear_paquetes(ruta);

    let address = format!("127.0.0.1:7666");
    println!("Conectado a ... {}", address);
    let mut stream = TcpStream::connect(address).expect("<CLIENTE>No me pude conectar al Banco!"); //Falta manejar el error

    for paquetes_turistico in paquetes_turisticos {
        enviar_pago_a_banco(&mut stream, paquetes_turistico.precio);
        leer_respuesta(&mut stream);
    }
}

fn enviar_pago_a_banco(stream: &mut TcpStream, monto: u32) {
    let cantidad_pago = monto.to_string();

    let size = (cantidad_pago.len() + 1) as u8;
    let buffer = [Message::Pay.into(), size]; //Me armo el buffer de aviso, primer byte tipo, segundo byte tamaño
    match stream.write_all(&buffer) {
        Ok(_) => {
            println!("<Cliente> Mensaje enviado correctamente!");
        }
        Err(_) => {
            println!("<Cliente> No me pude contactar con el banco!");
            // exit(0);
        }
    }

    let mut buffer_envio: Vec<u8> = Vec::with_capacity(size.into()); //Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut buffer_envio, cantidad_pago);
    match stream.write(&buffer_envio) {
        Ok(_) => {
            println!("<Cliente> Mensaje enviado correctamente!");
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
        Message::Succesfull => {
            println!("<Cliente> Pago procesado correctamente!\n");
        }
        Message::Error => {
            println!("<Cliente> No pudo procesarce el pago!\n");
        }
        _ => {
            println!("<Cliente>No se que me contesto el server!\n");
        }
    }
}

fn parsear_paquetes(ruta: &String) -> Vec<PaqueteTuristico> {
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
                    Err(e) => { eprintln!("Ups... {}", e) }
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    return paquetes_turisticos;
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
