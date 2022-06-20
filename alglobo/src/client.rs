use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::TcpStream;
use actix::{Actor, Context, Handler, System, Message};
use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull, Unknown};

const FILE: &str = "src/archivo.csv";

fn conectar_con_servidor(ip:&str, port:&str, tipo: String) -> Result<TcpStream, std::io::Error>{
    let address= format!("{}:{}", ip, port);
    println!("<CLIENTE> Intentando establecer conexión con:  {}",address);
    let mut stream = TcpStream::connect( address);
    stream
}

pub struct Banco {
    stream: Result<TcpStream, std::io::Error>,
    file: String,
}

impl Actor for Banco {
    type Context = Context<Self>;
}

impl Banco {
    pub fn new(ip:&str, port:&str, logger: String) -> Self {
        let bank_channel = conectar_con_servidor(ip, port, String::from("Bank"));
        Banco {
            stream: bank_channel,
            file: logger
        }
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
struct Prueba();

impl Handler<Prueba> for Banco {
    type Result = bool;

    fn handle(&mut self, msg: Prueba, _ctx: &mut Context<Self>) -> Self::Result {
        match &self.stream {
            Ok(stream) => {
                println!("Conectado exitosamente");
                true
            }
            Err(_) => {
                println!("No me pude conectar!");
                false
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
struct Procesar(PaqueteTuristico);

impl Handler<Procesar> for Banco {
    type Result = bool;

    fn handle(&mut self, msg: Procesar, _ctx: &mut Context<Self>) -> Self::Result {
        match &self.stream {
            Ok(stream) => {
                let mut channel = stream.try_clone().unwrap();
                println!("Conectado exitosamente");
                enviar_pago_a_banco(&mut channel, msg.0);
                leer_respuesta(&mut stream.try_clone().unwrap());
                true
            }
            Err(_) => {
                println!("No me pude conectar!");
                false
            }
        }
    }
}

#[allow(dead_code)]
struct PaqueteTuristico {
    id: u32,
    precio: u32,
    vuelo: String,
    hotel: String,
}

pub fn run(ip:&str) {

    let system = System::new();
    system.block_on(async {
        let banco_addr = Banco::new("127.0.0.1", "3001",String::from(FILE)).start();
        let resp = banco_addr.send(Prueba()).await;

        let paquetes_turisticos = parsear_paquetes(FILE);
        for paquete in paquetes_turisticos {
            let resp = banco_addr.do_send(Procesar(paquete));
        }

        System::current().stop();
    });
    system.run();
}
//Funciones auxiliareas despues capaz las movemos a otro lado!
fn enviar_pago_a_banco(stream: &mut TcpStream, paquete: PaqueteTuristico) {
    let cantidad_pago = paquete.precio.to_string();
    let size = (cantidad_pago.len() + 1) as u8;
    let buffer = [Pay.into(), size]; // Me armo el buffer de aviso, primer byte tipo, segundo byte tamaño
    match stream.write_all(&buffer) {
        Ok(_) => {
            println!(
                "<Cliente> Mensaje (id: {}) enviado correctamente!",
                paquete.id
            );
        } //Hardcodeado mientras que no haya archivo
        Err(_) => {
            println!("<Cliente> No me pude contactar con el banco!");
            // exit(0);
        }
    }

    let mut buffer_envio: Vec<u8> = Vec::with_capacity(size.into()); // Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut buffer_envio, cantidad_pago);
    match stream.write(&buffer_envio) {
        Ok(_) => {
            println!(
                "<Cliente> Mensaje (id: {}) enviado correctamente!",
                paquete.id
            );
        }
        Err(_) => {
            println!("<Cliente> No me pude contactar con el banco!");
        }
    }
}

fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);

    buffer.extend_from_slice(data.as_bytes());
}

pub fn leer_respuesta(stream: &mut TcpStream) {
    let mut num_buffer = [0u8; 2];
    let _aux = stream.read_exact(&mut num_buffer);
    match Tipo::from(num_buffer[0]) {
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

