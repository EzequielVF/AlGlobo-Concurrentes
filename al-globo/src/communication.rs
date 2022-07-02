use std::io::{Read, Write};
use std::net::TcpStream;

use crate::types::{ERROR, ServerResponse, Transaction, TransactionResult};

pub use self::Type::{Commit, Error, Pay, Rollback, Successful, Unknown};

/// Respuesta recibidas por parte del servidor
pub enum Type {
    /// Ocurrió un error en el servidor
    Error,
    /// Mensaje de Pago
    Pay,
    /// Respuesta de procesamiento exitoso
    Successful,
    /// Commit de transacción
    Commit,
    /// Rollback de transacción
    Rollback,
    /// Mensaje desconocido
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

/// Realiza la conexión al servidor indicado en ip y puerto (`port`).
/// Si fue exitosa, devuelve el socket creado,
/// si no, retorna el `Err`
pub fn connect_to_server(ip: &str, port: &str) -> Result<TcpStream, std::io::Error> {
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

/// Envía el paquete turístico (`package`) por el socket (`stream`)
pub fn send_package(stream: &mut TcpStream, transaction: Transaction, name: &str) {
    let package_price = transaction.precio.to_string();
    let size = (package_price.len() + 1) as u8;
    let buffer = [Pay.into(), size];

    match stream.write_all(&buffer) {
        Ok(_) => {
            println!(
                "<{}> Mensaje (id: {}) enviado correctamente!",
                name, transaction.id
            );
        }
        Err(_) => {
            println!("<{}> No me pude contactar con el banco!", name);
            // exit(0);
        }
    }

    let mut send_buffer: Vec<u8> = Vec::with_capacity(size.into()); // Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut send_buffer, package_price);

    match stream.write(&send_buffer) {
        Ok(_) => {
            println!(
                "<{}> Mensaje (id: {}) enviado correctamente!",
                name, transaction.id
            );
        }
        Err(_) => {
            println!("<{}> No me pude contactar con la entidad!", name);
        }
    }
}

/// Envía el resultado de la transacción al servidor,
/// indicando cuál mediante su id y la acción a ejecutar
/// ya sea *commit* o *rollback*.
pub fn send_transaction_result(stream: &mut TcpStream, trans_result: TransactionResult) {
    let size = (trans_result.transaction_id.len() + 1) as u8;
    let buffer = if trans_result.success {
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

#[doc(hidden)]
fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);
    buffer.extend_from_slice(data.as_bytes());
}

/// Recibe el resultado de la operación por parte del servicio solicitado
/// y mapea el resultado a `RequestState`
pub fn read_answer(stream: &mut TcpStream) -> ServerResponse {
    let mut num_buffer = [0u8; 2];
    let _aux = stream.read_exact(&mut num_buffer);
    let size = num_buffer[1];

    let mut buffer_packet: Vec<u8> = vec![0; size as usize];
    let _bytes_read = stream.read_exact(&mut buffer_packet);
    let mut aux = String::new();

    aux = read(buffer_packet);
    let mut response = ServerResponse {
        transaction_id : aux,
        response: false,
    };

    match Type::from(num_buffer[0]) {
        Successful => {
            response.response = true;
        }
        _ => {
        }
    }
    return response;
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

