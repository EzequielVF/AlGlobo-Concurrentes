use std::env::args;
use std::net::{TcpStream, UdpSocket};
use std::process::exit;
use std::sync::{Arc, Barrier, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use actix::prelude::*;
use actix::SyncArbiter;

use crate::communication::{connect_to_server, read_answer};
use crate::config::read_config;
use crate::leader::{id_to_dataaddr, LeaderElection, TIMEOUT};
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
mod leader;

fn receiver(stream: &mut TcpStream, rec: Receiver<Addr<TransactionManager>>, name: String) {
    println!("¡Antes del recv!");
    match rec.recv() {
        Ok(addr) => {
            loop {
                println!("Dentro del OK!");
                let server_response: ServerResponse = read_answer(stream.try_clone().unwrap());
                println!("La respuesta dice: {:?}, entity:{}", server_response, name.clone());
                let entity_answer = EntityAnswer {
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
            println!("No recibí addr: {}", e);
            exit(0);
        }
    }
}

// subir los thread arriba y correr lo que ahora está en el main en un system block
#[actix_rt::main]
async fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        eprintln!("¡Argumentos insuficientes!");
        eprintln!("Uso: <id-replica>");
        exit(1);
    }

    println!("Líder arranca ejecución");

    // Get configuration
    let config = Box::leak(Box::new(read_config("al-globo/src/config.json")));

    // Start Logger
    let path = config["alglobo"]["log_file"].as_str().unwrap();
    let logger = SyncArbiter::start(1, move || Logger::new(path));

    logger.do_send(Log("main".to_string(), "****************************".to_string()));
    logger.do_send(Log("main".to_string(), "Está escribiendo una réplica".to_string()));
    logger.do_send(Log("main".to_string(), "****************************".to_string()));

    // Create Streams
    let airline_stream = connect_to_server(config, "airline");
    let bank_stream = connect_to_server(config, "bank");
    let hotel_stream = connect_to_server(config, "hotel");

    let airline_sender;
    let (airline_s, airline_rec) = mpsc::channel();
    //let (airline_s, airline_rec) = mpsc::channel();
    match airline_stream {
        Ok(stream) => {
            logger.do_send(Log("main".to_string(), "¡Me pude conectar con la aerolínea!".to_string()));
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
        Err(_) => {
            logger.do_send(Log("main".to_string(), "[main] ¡No Me pude conectar con la aerolínea!".to_string()));
            airline_sender = None;
        }
    }

    let bank_sender;
    let (bank_s, bank_rec) = mpsc::channel();
    match bank_stream {
        Ok(stream) => {
            logger.do_send(Log("main".to_string(), "Me pude conectar con el Banco!".to_string()));

            let socket = stream.try_clone().unwrap();
            let bank_logger = logger.clone();
            let sender_address = SyncArbiter::start(1, move ||
                Sender::new("bank".to_string(),
                            socket.try_clone().unwrap(),
                            bank_logger.clone()));
            bank_sender = Some(sender_address);
            let mut socket_clone = stream.try_clone().unwrap();

            thread::spawn(move || receiver(&mut socket_clone, bank_rec, "bank".to_string()));
        }
        Err(_) => {
            logger.do_send(Log("main".to_string(), "¡No me pude conectar con el Banco!".to_string()));
            bank_sender = None;
        }
    }

    let (hotel_s, hotel_rec) = mpsc::channel();
    let hotel_sender;
    match hotel_stream {
        Ok(stream) => {
            logger.do_send(Log("main".to_string(), "Me pude conectar con el Hotel!".to_string()));
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
        Err(_) => {
            logger.do_send(Log("main".to_string(), "No me pude conectar con el Hotel!".to_string()));
            hotel_sender = None;
        }
    }

    // init Fails Writer
    let fails_writer_file = config["alglobo"]["fails_file"].as_str().unwrap();
    let fails_writer_logger = logger.clone();
    let writer = SyncArbiter::start(1, move ||
        Writer::new(fails_writer_file, fails_writer_logger.clone()));

    // init answers Writer
    let answers_file = config["alglobo"]["answers_file"].as_str().unwrap();
    let answers_writer_logger = logger.clone();
    let answers_writer = SyncArbiter::start(1, move ||
        Writer::new(answers_file, answers_writer_logger.clone()));

    // init transactions Writer
    let transactions_file = config["alglobo"]["transactions_hash_file"].as_str().unwrap();
    let transactions_writer_logger = logger.clone();
    let transactions_writer = SyncArbiter::start(1, move ||
        Writer::new(transactions_file, transactions_writer_logger.clone()));

    // init transaction manager
    let transaction_manager_logger = logger.clone();

    let answers_file = config["alglobo"]["answers_file"].as_str().unwrap();
    let transactions_file = config["alglobo"]["transactions_hash_file"].as_str().unwrap();
    let transaction_manager = SyncArbiter::start(1, move ||
        TransactionManager::new(transaction_manager_logger.clone(),
                                writer.clone(), answers_writer.clone(),
                                answers_file, transactions_writer.clone(), transactions_file));
    airline_s.send(transaction_manager.clone());
    bank_s.send(transaction_manager.clone());
    hotel_s.send(transaction_manager.clone());
    transaction_manager.do_send(SendAddr(airline_sender, hotel_sender, bank_sender));

    // init reader
    let transactions_file = config["alglobo"]["transactions_file"].as_str().expect("No encontre archivo de configuración!");
    let reader_logger = logger.clone();
    let reader = SyncArbiter::start(1, move ||
        Reader::new(transactions_file,
                    transaction_manager.clone(),
                    reader_logger.clone()));

    let my_id = argv[1].parse::<usize>().unwrap();
    // Todo
    //let flag = Arc::new(Mutex::new(false));
    let mut leader_election = LeaderElection::new(my_id, reader.clone());
    let socket = UdpSocket::bind(id_to_dataaddr(my_id)).unwrap();
    let mut buf = [0; 4];

    loop {
        /*{
            *flag.lock().unwrap() = false; //revisar
        }*/
        if leader_election.am_i_leader() {
            reader.send(ReadTransaction()).await;
            //let flag_aux = flag.clone();
            let socket_aux = socket.try_clone().unwrap();
            thread::spawn(move || {
                //while !(*flag_aux.lock().unwrap()) {
                loop {
                    socket_aux
                        .set_read_timeout(None)
                        .expect("socket timeout, problema!");
                    let (_size, from) = socket_aux.recv_from(&mut buf).unwrap();
                    thread::sleep(Duration::from_secs(3));
                    socket_aux
                        .send_to("PONG".as_bytes(), from)
                        .expect("socket send, problema al enviar en PONG");
                }
            });
            // This made it worked
            tokio::signal::ctrl_c().await.unwrap();
            println!("Ctrl-C received, shutting down");
            //barrier.wait();
            //System::current().stop();

            break;
        } else {
            let leader_id = leader_election.get_leader_id();

            socket
                .send_to("PING".as_bytes(), id_to_dataaddr(leader_id))
                .unwrap();
            socket.set_read_timeout(Some(TIMEOUT)).unwrap();

            if let Ok((_size, _from)) = socket.recv_from(&mut buf) {
                // todo
                println!("Me contestó el líder");
            } else {
                println!("No me contestó el líder");
                leader_election.find_new();
                println!("Saliendo de find_new");
            }
            println!("Saliendo de NO SOY LÍDER");
        }
    }
}
