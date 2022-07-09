use std::net::TcpStream;
use std::process::exit;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;

use actix::prelude::*;
use actix::SyncArbiter;

use crate::communication::{connect_to_server, read_answer};
use crate::config::read_config;
use crate::logger::{Log, Logger};
use crate::reader::{Reader, ReadTransaction};
use crate::sender::Sender;
use crate::transaction_manager::{ProcessEntityAnswer, SendAddr, TransactionManager};
use crate::types::{Answer, EntityAnswer, ServerResponse};
use crate::writer::Writer;

mod communication;
pub mod config;
pub mod logger;
mod sender;
mod transaction_manager;
mod writer;
mod types;
mod reader;


fn receiver(stream: &mut TcpStream, rec: Receiver<Addr<TransactionManager>>, name: String) {
    println!("Antes del recv!");
    match rec.recv() {
        Ok(addr) => {
            loop {
                println!("Dentro del OK!");
                let server_response: ServerResponse = read_answer(stream.try_clone().unwrap());
                println!("la respuesta dice: {:?}, entity:{}", server_response, name.clone());
                let mut entity_answer = EntityAnswer {
                    entity_name: name.clone(),
                    transaction_id: server_response.transaction_id,
                    answer: if server_response.response {
                        Answer::Ok
                    } else {
                        Answer::Failed
                    },
                };

                addr.do_send(ProcessEntityAnswer(entity_answer));
            }
        }
        Err(e) => {
            println!("No recibi addr: {}", e.to_string());
            exit(0);
        }
    }
}

// subir los thread arriba y correr lo que ahora esta en el main en un system block
#[actix_rt::main]
async fn main() {
//fn main() {

    // Get configuration
    let config = Box::leak(Box::new(read_config("al-globo/src/config.json")));

    // Start Logger
    let mut path = config["alglobo"]["log_file"].as_str().unwrap();
    let logger = SyncArbiter::start(1, move || Logger::new(path));

    // Create Streams
    let airline_stream = connect_to_server(config, "airline");
    let bank_stream = connect_to_server(config, "bank");
    let hotel_stream = connect_to_server(config, "hotel");

    let airline_sender;
    let (airline_s, airline_rec) = mpsc::channel();
    //let (airline_s, airline_rec) = mpsc::channel();
    match airline_stream {
        Ok(stream) => {
            logger.do_send(Log("Me pude conectar con la aerolinea!".to_string()));
            //let socket = Arc::new(Mutex::new(stream));
            let airline_logger = logger.clone();
            let socket = stream.try_clone().unwrap();
            let sender_addr = SyncArbiter::start(1, move ||
                Sender::new("airline".to_string(),
                            socket.try_clone().unwrap(),
                            airline_logger.clone()));
            airline_sender = Some(sender_addr);
            let mut socket_aux = stream.try_clone().unwrap();
            thread::spawn(move || receiver(&mut socket_aux, airline_rec, "airline".to_string()));
        }
        Err(e) => {
            logger.do_send(Log("NO Me pude conectar con la aerolinea!".to_string()));
            airline_sender = None;
        }
    }

    let bank_sender;
    let (bank_s, bank_rec) = mpsc::channel();
    match bank_stream {
        Ok(stream) => {
            logger.do_send(Log("Me pude conectar con el Banco!".to_string()));
            //let socket = Arc::new(Mutex::new(stream));
            let socket = stream.try_clone().unwrap();
            let bank_logger = logger.clone();
            let aux = SyncArbiter::start(1, move ||
                Sender::new("bank".to_string(),
                            socket.try_clone().unwrap(),
                            bank_logger.clone()));
            bank_sender = Some(aux);
            let mut socket_aux = stream.try_clone().unwrap();
            thread::spawn(move || receiver(&mut socket_aux, bank_rec, "bank".to_string()));
        }
        Err(e) => {
            logger.do_send(Log("NO Me pude conectar con el Banco!".to_string()));
            bank_sender = None;
        }
    }

    let (hotel_s, hotel_rec) = mpsc::channel();
    let hotel_sender;
    match hotel_stream {
        Ok(stream) => {
            logger.do_send(Log("Me pude conectar con  el Hotel!".to_string()));
            let socket = stream.try_clone().unwrap();
            let hotel_logger = logger.clone();
            let aux = SyncArbiter::start(1, move ||
                Sender::new("hotel".to_string(),
                            socket.try_clone().unwrap(),
                            hotel_logger.clone()));
            hotel_sender = Some(aux);
            let mut socket_aux = stream.try_clone().unwrap();
            thread::spawn(move || receiver(&mut socket_aux, hotel_rec, "hotel".to_string()));
        }
        Err(e) => {
            logger.do_send(Log("NO Me pude conectar con el Hotel!".to_string()));
            hotel_sender = None;
        }
    }

    // init Writer
    let writer_file = config["alglobo"]["writer_file"].as_str().unwrap();
    let writer_logger_address = logger.clone();
    let writer = SyncArbiter::start(1, move ||
        Writer::new(writer_file, writer_logger_address.clone()));

    // init transaction_manager
    let transaction_manager_logger = logger.clone();

    let transaction_manager = SyncArbiter::start(1, move ||
        TransactionManager::new(transaction_manager_logger.clone(),
                                writer.clone()));
    airline_s.send(transaction_manager.clone());
    bank_s.send(transaction_manager.clone());
    hotel_s.send(transaction_manager.clone());
    transaction_manager.do_send(SendAddr(airline_sender, hotel_sender, bank_sender));

    // init reader
    let transactions_file = config["alglobo"]["transactions_file"].as_str().expect("No encontre archivo de configuracion!");
    let reader_logger = logger.clone();
    let reader = SyncArbiter::start(1, move ||
        Reader::new(transactions_file,
                    transaction_manager.clone(),
                    reader_logger.clone()));

    reader.do_send(ReadTransaction());


    // This made it worked
    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    System::current().stop();
}

/*
  // Start Logger
    let mut path = config["alglobo"]["log_file"].as_str().unwrap();
    let logger = SyncArbiter::start(1, move || Logger::new(path));

    //Create Streams
    let airline_stream = connect_to_server(config, "airline");
    let bank_stream = connect_to_server(config, "bank");
    let hotel_stream = connect_to_server(config, "hotel");

    // Init senders
    let airline_sender_stream = airline_stream.try_clone().unwrap();
    let airline_logger = logger.clone();
    let airline_sender = SyncArbiter::start(1, move ||
        ExternalEntitySender::new("airline".to_string(),
                                  airline_sender_stream.try_clone().unwrap(),
                                  airline_logger.clone()));
*/