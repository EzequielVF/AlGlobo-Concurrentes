use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::TcpStream;
use actix::{Actor, Context, Handler, System, Message};

use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull};

const LOGFILE: &str = "src/archivo.csv";

fn conectar_con_servidor(ip:&str, port:&str, tipo: String) -> Result<TcpStream, std::io::Error>{
    let address= format!("{}:{}", ip, port);
    println!("<CLIENTE> Intentando establecer conexión con:  {}",address);
    let mut stream = TcpStream::connect( address);
    stream
}

pub struct Banco {
    stream: Result<TcpStream, std::io::Error>,
    loggerFile: String,
}

impl Actor for Banco {
    type Context = Context<Self>;
}

impl Banco {
    pub fn new(ip:&str, port:&str, logger: String) -> Self {
        let bank_channel = conectar_con_servidor(ip, port, String::from("Bank"));
        Banco {
            stream: bank_channel,
            loggerFile: logger
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

pub fn run(ip:&str) {

    let system = System::new();
    system.block_on(async {
        let banco_addr = Banco::new("127.0.0.1", "3001",String::from(LOGFILE)).start();
        let resp = banco_addr.send(Prueba()).await;
        


        System::current().stop();
    });
    system.run();
}

