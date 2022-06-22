use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};

use crate::logger::Logger;

pub use self::Tipo::{Error, Pay, Succesfull, Unknown};

static SERVER_ARGS: usize = 2;
const ERROR: u8 = 1;

pub fn run(ip: &str, port: &str, nombre: &str) -> std::io::Result<()> {
    let address = format!("{}:{}", ip, port);
    let logger = Arc::new(Mutex::new(Logger::new(String::from(nombre))));
    {
        logger.lock().unwrap().log(format!("Esperando clientes en: {}", address).as_str());
    }
    loop {
        let listener = TcpListener::bind(&address)?;
        let connection: (TcpStream, SocketAddr) = listener.accept()?;
        let mut client_stream = connection.0;
        let logger_clon = logger.clone();
        thread::Builder::new()
            .name("<<Cliente>>".into())
            .spawn(move || {
                println!("Se lanzo un cliente!.");
                read_packet_from_client(&mut client_stream, logger_clon);
            })
            .unwrap();
    }
}

fn read_packet_from_client(stream: &mut TcpStream, logger: Arc<Mutex<Logger>>) {
    loop {
        let mut num_buffer = [0u8; 2];
        match stream.read_exact(&mut num_buffer) {
            Ok(_) => {
                let message_type = num_buffer[0].into(); //Primer byte es el tipo de mensaje
                let size = num_buffer[1]; //El segundo es el tamaño

                let mut buffer_packet: Vec<u8> = vec![0; size as usize]; //Me creo un contenedor del tamaño q me dijeron
                let _aux = stream.read_exact(&mut buffer_packet); //Leo lo que me dijeron que lea
                match message_type {
                    Tipo::Pay => {
                        {
                            logger.lock().unwrap().log("<SERVER> Recibi un pago, voy a procesarlo!");
                        }
                        let aux = read_pay(buffer_packet);
                        if pago_es_correcto() {
                            println!("<SERVER> El Pago de {}$ fue recibido adecuadamente.", aux);
                            send_succesfull_message(stream);
                        } else {
                            println!(
                                "<SERVER> Tuvimos un problema al validar el pago de {}$.",
                                aux
                            );
                            send_error_message(stream);
                        }
                    }
                    _ => {
                        println!("<SERVER> Mensaje desconocido");
                    }
                }
            }
            Err(_) => {
                println!("<SERVER> El cliente se desconecto y cerro el stream.");
                break;
            }
        }
    }
}

fn procesamiento_aleatorio() {
    const FACTOR_TEMPORAL: u64 = 1;
    let ms = thread_rng().gen_range(1000, 3000);
    thread::sleep(Duration::from_millis(ms * FACTOR_TEMPORAL));
}

fn pago_es_correcto() -> bool {
    const UMBRAL_ERROR: i32 = 500;

    procesamiento_aleatorio();

    let valor = thread_rng().gen_range(0, 1000);

    valor > UMBRAL_ERROR
}

fn send_succesfull_message(stream: &mut TcpStream) {
    let buffer = [Succesfull.into(), 0_u8];
    match stream.write_all(&buffer) {
        Ok(_) => {
            println!("<SERVER> Mensaje enviado correctamente!");
        }
        Err(_) => {
            println!("<SERVER> Hubo un problema al intentar mandar un mensaje al cliente!")
        }
    }
}

fn send_error_message(stream: &mut TcpStream) {
    let buffer = [Error.into(), 0_u8];
    match stream.write_all(&buffer) {
        Ok(_) => {
            println!("<SERVER> Mensaje enviado correctamente!");
        }
        Err(_) => {
            println!("<SERVER> Hubo un problema al intentar mandar un mensaje al cliente!")
        }
    }
}

fn bytes2string(bytes: &[u8]) -> Result<String, u8> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str.to_owned()),
        Err(_) => Err(ERROR),
    }
}

fn read_pay(buffer_packet: Vec<u8>) -> String {
    let mut _index = 0_usize;

    let pago_size: usize = buffer_packet[(_index) as usize] as usize; //esto es asi porque los string en su primer byte tiene el tamaño, seguido del contenido
    _index += 1;

    bytes2string(&buffer_packet[_index..(_index + pago_size)]).unwrap()
}

pub enum Tipo {
    Error,
    Pay,
    Succesfull,
    Unknown,
}

impl From<u8> for Tipo {
    fn from(code: u8) -> Tipo {
        match code & 0xF0 {
            0x00 => Pay,
            0x10 => Succesfull,
            0x20 => Error,
            _ => Unknown,
        }
    }
}

impl From<Tipo> for u8 {
    fn from(code: Tipo) -> u8 {
        match code {
            Pay => 0x00,
            Succesfull => 0x10,
            Error => 0x20,
            _ => 0x99,
        }
    }
}
