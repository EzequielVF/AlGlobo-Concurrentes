use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

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

const DEFAULT_PORT: &str = "7666";
const DEFAULT_IP: &str = "127.0.0.1";

pub fn get_address() -> String {
    format!("{}:{}", DEFAULT_IP, DEFAULT_PORT)
}

fn main() {
    let address = get_address();
    println!("Conectado a ... {}", &address);
    let mut stream = TcpStream::connect(address).unwrap(); //Falta manejar el error
    loop {
        mandar_pago(&mut stream);
        leer_respuesta(&mut stream);
    }
}

fn push_to_buffer(buffer: &mut Vec<u8>, data: String) {
    buffer.push(data.len() as u8);
    let data_bytes = data.as_bytes();
    for i in 0..data_bytes.len() {
        buffer.push(data_bytes[i]);
    }
}

pub fn mandar_pago(stream: &mut TcpStream) {
    //esto esta hardcodeado pero habria que pasar aca el mensaje que leemos del archivo
    let cantidad_pago = String::from("500");
    let size = (cantidad_pago.len()+1) as u8;
    let buffer = [Message::Pay.into(), size]; //Me armo el buffer de aviso, primer byte tipo, segundo byte tamaño
    stream.write_all(&buffer).unwrap(); //Aca le avise

    let mut buffer_envio: Vec<u8> = Vec::with_capacity(size.into()); //Aca me armo el buffer con el contenido del mensaje, en este caso solo me meto los "500" que quiero pagar
    push_to_buffer(&mut buffer_envio, cantidad_pago);
    stream.write(&buffer_envio).unwrap();
}

pub fn leer_respuesta(stream: &mut TcpStream) {
    let mut num_buffer = [0u8; 2];
    let _aux = stream.read_exact(&mut num_buffer);
    match Message::from(num_buffer[0]) {
        Message::Succesfull => {
            println!("\n<Cliente> Pago procesado correctamente!\n");
        }
        Message::Error => {
            println!("\n<Cliente> No pudo procesarce el pago!\n");
        }
        _ => {
            println!("\nNo se que me contesto el server!\n");
        }
    }
}