use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use rand::thread_rng;
use rand::Rng;


static SERVER_ARGS: usize = 2;

const DEFAULT_PORT: &str = "7666";
const DEFAULT_IP: &str = "127.0.0.1";
const ERROR: u8 = 1;

pub enum Message {
    Pay,
    Succesfull,
    Error,
    Unknown
}

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

pub fn get_address() -> String {
    format!("{}:{}", DEFAULT_IP, DEFAULT_PORT)
}

fn main() {
    let address = get_address();
    println!("IP: {}", &address);
    println!("Esperando clientes...");
    wait_new_clients(&address);
}

fn wait_new_clients(address: &str) -> std::io::Result<()> {
    loop {
        let listener = TcpListener::bind(&address)?;
        let connection: (TcpStream, SocketAddr) = listener.accept()?;
        let mut client_stream = connection.0;
        thread::Builder::new()
            .name("<<Cliente-Banco>>".into())
            .spawn(move || {
                println!("Se lanzo un cliente!.");
                handle_client(&mut client_stream);
            })
            .unwrap();
    }
}

fn handle_client(stream: &mut TcpStream) {
    let mut stream_cloned = stream.try_clone().unwrap();
    read_packet_from_client(&mut stream_cloned);
}

fn procesamiento_aleatorio() {
    const FACTOR_TEMPORAL: u64 = 3;
    let ms = thread_rng().gen_range(1000, 3000);
    thread::sleep(Duration::from_millis(ms * FACTOR_TEMPORAL));
}

fn pago_es_correcto() -> bool {
    procesamiento_aleatorio();
    let valor = thread_rng().gen_range(0, 1000);
    if valor > 500 {
        true
    } else {
        false
    }
}

fn send_succesfull_message(stream: &mut TcpStream) {
    let buffer = [Message::Succesfull.into(), 0_u8];
    stream.write_all(&buffer).unwrap();
}

fn send_error_message(stream: &mut TcpStream) {
    let buffer = [Message::Error.into(), 0_u8];
    stream.write_all(&buffer).unwrap();
}

fn bytes2string(bytes: &[u8]) -> Result<String, u8> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str.to_owned()),
        Err(_) => Err(ERROR),
    }
}

fn read_pay(buffer_packet: Vec<u8>) -> String {
    let mut _index = 0 as usize;
    let mut _aux: String;
    let pago_size: usize = buffer_packet[(_index) as usize] as usize; //esto es asi porque los string en su primer byte tiene el tamaño, seguido del contenido
    _index += 1 as usize;
    _aux = bytes2string(&buffer_packet[_index..(_index + pago_size)]).unwrap(); //falta manejar
    _aux
}

fn read_packet_from_client(stream: &mut TcpStream) {
    loop {
        let mut num_buffer = [0u8; 2];
        match stream.read_exact(&mut num_buffer) {
            Ok(_) => {
                let message_type = num_buffer[0].into(); //Primer byte es el tipo de mensaje
                let size = num_buffer[1]; //El segundo es el tamaño

                let mut buffer_packet: Vec<u8> = vec![0; size as usize];//Me creo un contenedor del tamaño q me dijeron
                let _aux = stream.read_exact(&mut buffer_packet); //Leo lo que me dijeron que lea
                match message_type{
                    Message::Pay => {
                        println!("<BANCO> Recibi un pago, voy a procesarlo!");
                        let aux = read_pay(buffer_packet);
                        if pago_es_correcto() {
                            send_succesfull_message(stream);
                            println!("<BANCO> El Pago de {}$ fue recibido adecuadamente.", aux);
                        } else {
                            send_error_message(stream);
                            println!("<BANCO> Tuvimos un problema al validar el pago de {}$.", aux);
                        }
                    }
                    _ => {
                        println!("<BANCO> Mensaje desconocido");
                    }
                }
            }
            Err(_) => {
                println!("El cliente se desconecto y cerro el stream.");
                break;
            }
        }
    }
}
