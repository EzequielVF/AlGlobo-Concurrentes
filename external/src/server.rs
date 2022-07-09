use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rand::{thread_rng, Rng};

use crate::logger::Logger;
use crate::server::Type::{Commit, Rollback};

pub use self::Type::{Error, Pay, Successful, Unknown};

const ERROR: u8 = 1;

pub fn run(ip: &str, port: &str, nombre: &str) -> std::io::Result<()> {
    let address = format!("{}:{}", ip, port);
    let logger = Arc::new(Mutex::new(Logger::new(nombre)));
    {
        logger
            .lock()
            .unwrap()
            .log(format!("Esperando clientes en: {}", address).as_str());
    }
    loop {
        let listener = TcpListener::bind(&address)?;
        let connection: (TcpStream, SocketAddr) = listener.accept()?;
        let mut client_stream = connection.0;
        let logger_clone = logger.clone();
        thread::Builder::new()
            .name("<<Cliente>>".into())
            .spawn(move || {
                logger_clone.lock().unwrap().log("Se lanzó un cliente!");
                read_packet_from_client(&mut client_stream, logger_clone);
            })
            .unwrap();
    }
}

fn read_packet_from_client(stream: &mut TcpStream, logger: Arc<Mutex<Logger>>) {
    loop {
        let mut num_buffer = [0u8; 2];
        match stream.read_exact(&mut num_buffer) {
            Ok(_) => {
                let message_type = num_buffer[0].into(); // Primer byte es el tipo de mensaje
                let size = num_buffer[1]; // El segundo es el tamaño

                let mut buffer_packet: Vec<u8> = vec![0; size as usize]; // Me creo un contenedor del tamaño q me dijeron
                let _bytes_read = stream.read_exact(&mut buffer_packet); // Leo lo que me dijeron que lea
                let mut aux = String::new();
                match message_type {
                    Pay => {
                        aux = read(buffer_packet);
                        logger.lock().unwrap().log(
                            format!(
                                "<SERVER> Recibí una transacción de código {}, voy a procesarlo!",
                                aux
                            )
                            .as_str(),
                        );
                        if successful_payment() {
                            logger.lock().unwrap().log(format!("<SERVER> El Pago de {}$ fue recibido adecuadamente.", aux).as_str());
                            send_message(stream, aux, true, logger.clone());
                        } else {
                            logger.lock().unwrap().log(format!("<SERVER> Tuvimos un problema al validar el pago de {}$.", aux).as_str());
                            send_message(stream, aux, false, logger.clone());
                        }
                    }
                    Commit => {
                        let aux = read(buffer_packet);
                        logger.lock().unwrap().log(
                            format!("<SERVER> La operación con ID:{} fue commiteada", aux).as_str(),
                        );
                    }
                    Rollback => {
                        let aux = read(buffer_packet);
                        logger.lock().unwrap().log(
                            format!("<SERVER> La operación con ID:{} fue rollbackeada!", aux)
                                .as_str(),
                        );
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

fn random_duration_processing() {
    const FACTOR_TEMPORAL: u64 = 0;
    let ms = thread_rng().gen_range(2000, 5000);
    thread::sleep(Duration::from_millis(ms * FACTOR_TEMPORAL));
}

fn successful_payment() -> bool {
    const ERROR_THRESHOLD: i32 = 1001;

    random_duration_processing();

    let random_value = thread_rng().gen_range(0, 1000);

    //random_value > ERROR_THRESHOLD
    true
}

#[doc(hidden)]
fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);
    buffer.extend_from_slice(data.as_bytes());
}

fn send_message(stream: &mut TcpStream, id: String, estado: bool, logger: Arc<Mutex<Logger>>) {
    let size = (id.len() + 1) as u8;
    let mut buffer = [Error.into(), size];
    if estado {
        buffer = [Successful.into(), size];
    }
    match stream.write_all(&buffer) {
        Ok(_) => {
            logger.lock().unwrap().log(format!("Se envió cabecera para id {} al cliente!", id).as_str());
        }
        Err(_) => {
            logger.lock().unwrap().log((format!(
                "Hubo un problema al intentar enviar cabecera para transacción de id {} al cliente!", id).as_str()));
        }
    }

    let mut send_buffer: Vec<u8> = Vec::with_capacity(size.into());
    push_to_buffer(&mut send_buffer, id.clone());
    match stream.write(&send_buffer) {
        Ok(_) => {
            logger.lock().unwrap().log(format!("Mensaje (id: {}) enviado correctamente!", id).as_str());
        }
        Err(_) => {
            logger.lock().unwrap().log(format!("No pude enviar respuesta para transacción de id {}", id).as_str());
        }
    }
}

fn bytes2string(bytes: &[u8]) -> Result<String, u8> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str.to_owned()),
        Err(_) => Err(ERROR),
    }
}

fn read(buffer_packet: Vec<u8>) -> String {
    let mut _index = 0_usize;

    let pago_size: usize = buffer_packet[(_index) as usize] as usize; // esto es asi porque los string en su primer byte tiene el tamaño, seguido del contenido
    _index += 1;

    bytes2string(&buffer_packet[_index..(_index + pago_size)]).unwrap()
}

pub enum Type {
    Error,
    Pay,
    Successful,
    Commit,
    Rollback,
    Unknown,
}

impl From<u8> for Type {
    fn from(code: u8) -> Type {
        match code & 0xF0 {
            0x00 => Pay,
            0x10 => Successful,
            0x20 => Error,
            0x30 => Commit,
            0x40 => Rollback,
            _ => Unknown,
        }
    }
}

impl From<Type> for u8 {
    fn from(code: Type) -> u8 {
        match code {
            Pay => 0x00,
            Successful => 0x10,
            Error => 0x20,
            Commit => 0x30,
            Rollback => 0x40,
            _ => 0x99,
        }
    }
}
