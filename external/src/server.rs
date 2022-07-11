use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};

use crate::logger::Logger;
use crate::server::Type::{Commit, Rollback};

pub use self::Type::{Error, Pay, Successful, Unknown};

const ERROR: u8 = 1;

pub enum TransactionStatus {
    Prepare,
    Commit,
    Abort,
    Failed,
}

pub fn run(ip: &str, port: &str, nombre: &str, success_rate: u8) -> std::io::Result<()> {
    let address = format!("{}:{}", ip, port);
    let logger = Arc::new(Mutex::new(Logger::new(nombre)));
    /*{
        logger
            .lock()
            .expect("No pude tomar al logger porque esta envenenado!")
            .log(format!("Esperando clientes en: {}", address).as_str());
    }*/
    do_log(logger.clone(), format!("Esperando clientes en: {}", address));
    let transactions = Arc::new(RwLock::new(HashMap::<String, TransactionStatus>::new()));

    loop {
        let listener = TcpListener::bind(&address)?;
        let connection: (TcpStream, SocketAddr) = listener.accept()?;
        let mut client_stream = connection.0;
        let transactions_clone = transactions.clone();

        let logger_clone = logger.clone();
        match thread::Builder::new()
            .name("<<Servidor>>".into())
            .spawn(move || {
                logger_clone.lock().unwrap().log("Se lanzó un cliente!");
                read_packet_from_client(&mut client_stream, logger_clone, transactions_clone.clone(), success_rate);
            }) {
            Ok(_) => {
                println!("Hilo para atender al cliente abierto correctamente!");
            }
            Err(_) => {
                println!("Problema al lanzar hilo para atender al cliente!");
            }
        }
    }
}

fn read_packet_from_client(stream: &mut TcpStream, logger: Arc<Mutex<Logger>>, transactions_arc: Arc<RwLock<HashMap<String, TransactionStatus>>>, success_rate: u8) {
    loop {
        let mut num_buffer = [0u8; 2];
        match stream.read_exact(&mut num_buffer) {
            Ok(_) => {
                let message_type = num_buffer[0].into(); // Primer byte es el tipo de mensaje
                let size = num_buffer[1]; // El segundo es el tamaño

                let mut buffer_packet: Vec<u8> = vec![0; size as usize]; // Me creo un contenedor del tamaño q me dijeron
                let _bytes_read = stream.read_exact(&mut buffer_packet); // Leo lo que me dijeron que lea

                match message_type {
                    Pay => {
                        let (id, precio) = read_pay(buffer_packet);
                        //logger.lock().unwrap().log(format!("Recibí una transacción de código {} y precio: {}, voy a procesarlo!", id, precio).as_str());
                        do_log(logger.clone(), format!("Recibí una transacción de código {} y precio: {}, voy a procesarlo!", id, precio));


                        // Ver si ya lo habia procesados -> lock del hash, me fijo si ya lo procese.
                        if let Ok(transactions) = transactions_arc.read() {
                            if let Some(state) = transactions.get(&id) {
                                match state {
                                    TransactionStatus::Commit | TransactionStatus::Prepare => {
                                        do_log(logger.clone(), format!("La transacción id {} fue procesada exitosamente", id));
                                        send_message(stream, id.clone(), true, logger.clone());
                                    }
                                    TransactionStatus::Failed | TransactionStatus::Abort => {
                                        do_log(logger.clone(), format!("Tuvimos un problema validando la transacción id: {}", id));
                                        send_message(stream, id.clone(), false, logger.clone());
                                    }
                                }
                                continue;
                            }
                        }
                        if successful_payment(success_rate) {
                            //logger.lock().unwrap().log(format!("La transacción id {} fue procesada exitosamente", id).as_str());
                            do_log(logger.clone(), format!("La transacción id {} fue procesada exitosamente", id));
                            if let Ok(mut transactions) = transactions_arc.write() {
                                transactions.insert(id.clone(), TransactionStatus::Prepare);
                            }
                            send_message(stream, id.clone(), true, logger.clone());
                        } else {
                            do_log(logger.clone(), format!("Tuvimos un problema validando la transacción id: {}", id));
                            if let Ok(mut transactions) = transactions_arc.write() {
                                transactions.insert(id.clone(), TransactionStatus::Failed);
                            }
                            send_message(stream, id.clone(), false, logger.clone());
                        }
                    }
                    Commit => {
                        let id = read(buffer_packet);
                        /*logger.lock().unwrap().log(
                            format!("La operación con ID: {} fue commiteada", id).as_str(),
                        );*/
                        do_log(logger.clone(), format!("La operación con ID: {} fue commiteada", id));
                        if let Ok(mut transactions) = transactions_arc.write() {
                            transactions.insert(id.clone(), TransactionStatus::Commit);
                        }
                    }
                    Rollback => {
                        let id = read(buffer_packet);
                        /*logger.lock().unwrap().log(
                            format!("La operación con ID:{} fue rollbackeada!", id)
                                .as_str(),
                        );*/
                        do_log(logger.clone(), format!("La operación con ID:{} fue rollbackeada!", id));
                        if let Ok(mut transactions) = transactions_arc.write() {
                            transactions.insert(id.clone(), TransactionStatus::Abort);
                        }
                    }
                    _ => {
                        println!("Mensaje desconocido");
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
    const FACTOR_TEMPORAL: u64 = 1;
    let ms = thread_rng().gen_range(5000, 10000);
    thread::sleep(Duration::from_millis(ms * FACTOR_TEMPORAL));
}

fn successful_payment(success_rate: u8) -> bool {
    random_duration_processing();

    let random_value = thread_rng().gen_range(0, 100);

    random_value < success_rate
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
            //logger.lock().unwrap().log(format!("Se envió cabecera para id {} al cliente!", id).as_str());
            do_log(logger.clone(), format!("Se envió cabecera para id {} al cliente!", id));
        }
        Err(_) => {
            /*logger.lock().unwrap().log(format!(
                "Hubo un problema al intentar enviar cabecera para transacción de id {} al cliente!", id).as_str());*/
            do_log(logger.clone(), format!(
                "Hubo un problema al intentar enviar cabecera para transacción de id {} al cliente!", id));
        }
    }

    let mut send_buffer: Vec<u8> = Vec::with_capacity(size.into());
    push_to_buffer(&mut send_buffer, id.clone());
    match stream.write(&send_buffer) {
        Ok(_) => {
            //logger.lock().unwrap().log(format!("Mensaje (id: {}) enviado correctamente!", id).as_str());
            do_log(logger.clone(), format!("Mensaje (id: {}) enviado correctamente!", id));
        }
        Err(_) => {
            //logger.lock().unwrap().log(format!("No pude enviar respuesta para transacción de id {}", id).as_str());
            do_log(logger.clone(), format!("No pude enviar respuesta para transacción de id {}", id));
        }
    }
}

fn bytes2string(bytes: &[u8]) -> Result<String, u8> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str.to_owned()),
        Err(_) => Err(ERROR),
    }
}

fn read_pay(buffer_packet: Vec<u8>) -> (String, String) {
    let mut _index = 0 as usize;

    let id_size: usize = buffer_packet[(_index) as usize] as usize;
    _index += 1 as usize;
    let aux_1 ;
    //= bytes2string(&buffer_packet[_index..(_index + id_size)]).unwrap();

    match bytes2string(&buffer_packet[_index..(_index + id_size)]) {
        Ok(message) => { aux_1 = message }
        Err(_) => { aux_1 = "INDEFINIDO".to_string() }
    }
    _index += id_size;

    // TODO ¿Por qué dejó de usarse esta variable?
    let _pago_size: usize = buffer_packet[(_index) as usize] as usize;
    _index += 1 as usize;

    let aux_2 ;
    //= bytes2string(&buffer_packet[_index..(_index + pago_size)]).unwrap();

    match bytes2string(&buffer_packet[_index..(_index + id_size)]) {
        Ok(message) => { aux_2 = message }
        Err(_) => { aux_2 = "INDEFINIDO".to_string() }
    }
    (aux_1, aux_2)
}

fn read(buffer_packet: Vec<u8>) -> String {
    let mut _index = 0 as usize;

    let pago_size: usize = buffer_packet[(_index) as usize] as usize; // esto es asi porque los string en su primer byte tiene el tamaño, seguido del contenido
    _index += 1;

    match bytes2string(&buffer_packet[_index..(_index + pago_size)]) {
        Ok(message) => { message }
        Err(_) => { "INDEFINIDO".to_string() }
    }

    //bytes2string(&buffer_packet[_index..(_index + pago_size)]).unwrap()
}

fn do_log(logger: Arc<Mutex<Logger>>, text: String) {
    match logger.lock() {
        Ok(mut guard) => {
            guard.log(text.as_str());
        }
        Err(_) => {
            println!("El mutex del logger esta Envenenado!");
        }
    };
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
