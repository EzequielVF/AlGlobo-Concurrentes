use std::io::{Read, Write};
use std::net::TcpStream;

use crate::external_entity::TransactionResult;
use crate::payment_processor::RequestState;
use crate::PaqueteTuristico;

pub use self::Tipo::{Commit, Error, Pay, Rollback, Successful, Unknown};

pub enum Tipo {
    Error,
    Pay,
    Successful,
    Commit,
    Rollback,
    Unknown,
}

impl From<u8> for Tipo {
    fn from(code: u8) -> Tipo {
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

impl From<Tipo> for u8 {
    fn from(code: Tipo) -> u8 {
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

// LOGICA SERVER
pub fn conectar_con_servidor(ip: &str, port: &str) -> Result<TcpStream, std::io::Error> {
    let address = format!("{}:{}", ip, port);
    println!("<CLIENTE> Intentando establecer conexión con: {}", address);
    let stream = TcpStream::connect(address);
    match stream {
        Ok(stream) => {
            println!("Conectado exitosamente");
            Ok(stream)
        }
        Err(e) => {
            println!("No me pude conectar!");
            Err(e)
        }
    }
}

pub fn enviar_paquete(stream: &mut TcpStream, paquete: PaqueteTuristico, name: &str) {
    let cantidad_pago = paquete.precio.to_string();
    let size = (cantidad_pago.len() + 1) as u8;
    let buffer = [Pay.into(), size];
    match stream.write_all(&buffer) {
        Ok(_) => {
            println!(
                "<{}> Mensaje (id: {}) enviado correctamente!",
                name, paquete.id
            );
        }
        Err(_) => {
            println!("<{}> No me pude contactar con el banco!", name);
            // exit(0);
        }
    }
    let mut buffer_envio: Vec<u8> = Vec::with_capacity(size.into()); // Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut buffer_envio, cantidad_pago);
    match stream.write(&buffer_envio) {
        Ok(_) => {
            println!(
                "<{}> Mensaje (id: {}) enviado correctamente!",
                name, paquete.id
            );
        }
        Err(_) => {
            println!("<{}> No me pude contactar con el banco!", name);
        }
    }
}

pub fn enviar_resultado(stream: &mut TcpStream, trans_result: TransactionResult) {
    let size = (trans_result.transaction_id.len() + 1) as u8;
    let buffer = if trans_result.result {
        [Commit.into(), size]
    } else {
        [Rollback.into(), size]
    };

    match stream.write_all(&buffer) {
        Ok(_) => {
            println!(
                "Mensaje (id: {}) enviado correctamente!",
                trans_result.transaction_id
            );
        }
        Err(_) => {
            println!("No me pude contactar!");
        }
    }
    let mut buffer_envio: Vec<u8> = Vec::with_capacity(size.into()); // Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut buffer_envio, trans_result.transaction_id.clone());
    match stream.write(&buffer_envio) {
        Ok(_) => {
            println!(
                "Mensaje (id: {}) enviado correctamente!",
                trans_result.transaction_id
            );
        }
        Err(_) => {
            println!(" No me pude contactar!");
        }
    }
}

fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);

    buffer.extend_from_slice(data.as_bytes());
}

pub fn leer_respuesta(stream: &mut TcpStream) -> RequestState {
    let mut num_buffer = [0u8; 2];
    let _aux = stream.read_exact(&mut num_buffer);
    match Tipo::from(num_buffer[0]) {
        Successful => RequestState::Ok,
        Error => RequestState::Failed,
        _ => RequestState::Failed,
    }
}
