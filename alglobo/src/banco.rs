use std::io::{Read, Write};
use std::net::TcpStream;
use actix::{Actor, Context, Handler, System, Message};
use crate::comunicacion::{Pay, Tipo};
use crate::client::PaqueteTuristico;

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
pub struct Prueba();

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
pub struct Procesar(pub PaqueteTuristico);

impl Handler<Procesar> for Banco {
    type Result = bool;

    fn handle(&mut self, msg: Procesar, _ctx: &mut Context<Self>) -> Self::Result {
        match &self.stream {
            Ok(stream) => {
                let mut channel = stream.try_clone().unwrap();
                println!("<Cliente> Conexion estable...");
                enviar_pago_a_banco(&mut channel, msg.0);
                leer_respuesta(&mut stream.try_clone().unwrap());
                true
            }
            Err(_) => {
                println!("<Cliente> La conexion no fue establecida!");
                false
            }
        }
    }
}

fn conectar_con_servidor(ip:&str, port:&str, tipo: String) -> Result<TcpStream, std::io::Error>{
    let address= format!("{}:{}", ip, port);
    println!("<CLIENTE> Intentando establecer conexión con:  {}",address);
    let mut stream = TcpStream::connect( address);
    stream
}

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
        }
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