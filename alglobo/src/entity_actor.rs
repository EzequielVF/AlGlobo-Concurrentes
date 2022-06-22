use std::io::{Read, Write};
use std::net::TcpStream;
use actix::{Actor, Context, Handler, Message, System};
use crate::client::PaqueteTuristico;
use crate::comunicacion::{conectar_con_servidor, enviar_paquete, leer_respuesta, Pay, Tipo};

pub struct EntityActor {
    stream: Result<TcpStream, std::io::Error>,
    file: String,
}

impl Actor for EntityActor {
    type Context = Context<Self>;
}

impl EntityActor {
    pub fn new(ip: &str, port: &str, logger: String) -> Self {
        let bank_channel = conectar_con_servidor(ip, port);
        EntityActor {
            stream: bank_channel,
            file: logger,
        }
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct ConnectionStatus();

impl Handler<ConnectionStatus> for EntityActor {
    type Result = bool;

    fn handle(&mut self, msg: ConnectionStatus, _ctx: &mut Context<Self>) -> Self::Result {
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
pub struct ProcesarPaquete(pub PaqueteTuristico);

impl Handler<ProcesarPaquete> for EntityActor {
    type Result = bool;

    fn handle(&mut self, msg: ProcesarPaquete, _ctx: &mut Context<Self>) -> Self::Result {
        match &self.stream {
            Ok(stream) => {
                let mut channel = stream.try_clone().unwrap();
                enviar_paquete(&mut channel, msg.0);
                //leer_respuesta(&mut stream.try_clone().unwrap());
                true
            }
            Err(_) => {
                println!("<Cliente> La conexion no fue establecida!");
                false
            }
        }
    }
}
