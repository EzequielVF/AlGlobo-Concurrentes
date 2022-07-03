use crate::communication::connect_to_server;
use actix::prelude::*;
use std::collections::HashMap;
use std::env::args;
use std::io::Error;
use std::net::TcpStream;
use std::process::exit;
use std::sync::mpsc;

use crate::config::Configuration;
use crate::external_entity_receiver::ExternalEntityReceiver;
use crate::external_entity_sender::ExternalEntitySender;
use crate::logger::{Log, Logger};
use crate::reader::{ParseTouristPackage, Reader};
use crate::transaction_manager::TransactionManager;
use crate::writer::Writer;

mod communication;
mod config;
mod external_entity_receiver;
mod external_entity_sender;
mod logger;
mod reader;
mod transaction_manager;
mod types;
mod writer;

// fn run_logger() -> Addr<Logger> {
//     // Create channel to comunicate with logger
//     let (logger_s, logger_rec) = mpsc::channel();
//     // Run logger in a new Arbiter
//     let logger_arbiter = Arbiter::new();
//     let logger_execution = async move {
//         let logger_address = Logger::new("test.log").start();
//         logger_s
//             .send(logger_address)
//             .expect("logger send, problema al enviar!");
//     };
//     logger_arbiter.spawn(logger_execution);
//     let logger_address = logger_rec.recv().unwrap();
//     logger_address
// }
//
// fn run_receiver(name: &'static str, stream: Result<&'static TcpStream, &'static Error>, logger_address: Addr<Logger>) -> Addr<ExternalEntityReceiver> {
//     // ********** Run Airline Receiver**********
//     let (rec_s, rec_rec) = mpsc::channel();
//     // Run airline in a new Arbiter
//     let arbiter = Arbiter::new();
//     let execution = async move {
//         let address =
//             ExternalEntityReceiver::new(name, stream, logger_address)
//                 .start();
//         rec_s
//             .send(address)
//             .expect("problema al enviar address!");
//     };
//     arbiter.spawn(execution);
//     let address = rec_rec.recv().unwrap();
//     address
// }
//
// fn run_sender(name: &'static str, stream: Result<&'static TcpStream, &'static Error>, logger_address: Addr<Logger>, receiver_addr: Addr<ExternalEntityReceiver>) -> Addr<ExternalEntitySender> {
//     let (rec_s, rec_rec) = mpsc::channel();
//     let arbiter = Arbiter::new();
//     let execution = async move {
//         let address = ExternalEntitySender::new(name, stream, logger_address, receiver_addr)
//             .start();
//         rec_s
//             .send(address)
//             .expect("airline send, problema al enviar!");
//     };
//     arbiter.spawn(execution);
//     let address = rec_rec.recv().unwrap();
//     address
// }
//
// fn run_writer(name: &str, logger_address: Addr<Logger>) -> Addr<Writer> {
//     let (writer_s, writer_rec) = mpsc::channel();
//     let arbiter = Arbiter::new();
//     let execution = async move {
//         let address =
//             Writer::new("failed-transactions", logger_address)
//                 .start();
//         writer_s
//             .send(address)
//             .expect("writer send, problema al enviar!");
//     };
//     arbiter.spawn(execution);
//     let address = writer_rec.recv().unwrap();
//     address
// }
//
// fn run_transaction_manager(airline_sender: Addr<ExternalEntitySender>, bank_sender: Addr<ExternalEntitySender>,
//                            hotel_sender: Addr<ExternalEntitySender>, logger: Addr<Logger>, writer: Addr<Writer>) -> Addr<TransactionManager> {
//     let (tx, rx) = mpsc::channel();
//     let arbiter = Arbiter::new();
//     let execution = async move {
//         let address = TransactionManager::new(
//             airline_sender.clone(),
//             bank_sender.clone(),
//             hotel_sender.clone(),
//             logger.clone(),
//             writer.clone(),
//         )
//             .start();
//         tx.send(address).expect("pp send, problema al enviar!");
//     };
//     let address = rx.recv().unwrap();
//     address
// }
//
// fn run_reader(path: &str, tm_addr: Addr<TransactionManager>, logger_addr: Addr<Logger>) {
//
// }

/*
 * init_app() -> Result<(), &'static str> {
 * let logger_address = run_logger();
 *
 * // Create Streams
 * let airline_stream: Result<&TcpStream, &Error> = connect_to_server("127.0.0.1", "3000");
 * let bank_stream: Result<&TcpStream, &Error> = connect_to_server("127.0.0.1", "3001");
 * let hotel_stream: Result<&TcpStream, &Error> = connect_to_server("127.0.0.1", "3002");
 *
 * let airline_receiver = run_receiver("AIRLINE", airline_stream, logger_address.clone());
 * let bank_receiver = run_receiver("BANK", bank_stream, logger_address.clone());
 * let hotel_receiver = run_receiver("HOTEL", hotel_stream, logger_address.clone());
 *
 * let airline_sender = run_sender("AIRLINE", airline_stream, logger_address.clone(), airline_receiver);
 * let bank_sender = run_sender("BANK", bank_stream, logger_address.clone(), bank_receiver);
 * let hotel_sender = run_sender("HOTEL", hotel_stream, logger_address.clone(), hotel_receiver);
 *
 * let writer_addr = run_writer("fallas.csv", logger_address.clone());
 * let tm_addr = run_transaction_manager(airline_sender, bank_sender, hotel_sender, logger_address.clone(), writer_addr);
 *
 * run_reader("archivo.csv", tm_addr, logger_address);
 *
 * Ok(())
 * }
 */

#[actix_rt::main]
async fn main() -> Result<(), &'static str> {
    //Run Logger
    let (logger_s, logger_rec) = mpsc::channel();
    let logger_arbiter = Arbiter::new();
    let logger_execution = async move {
        let logger_address = Logger::new("test").start();
        logger_s
            .send(logger_address)
            .expect("logger send, problema al enviar!");
    };
    logger_arbiter.spawn(logger_execution);
    let logger_address = logger_rec.recv().unwrap();


    // Create Streams
    let airline_stream = connect_to_server("127.0.0.1", "4000")?;
    let bank_stream = connect_to_server("127.0.0.1", "4001")?;
    let hotel_stream = connect_to_server("127.0.0.1", "4002")?;


    // ********** Run Airline Receiver **********
    let (airline_rec_s, airline_rec_rec) = mpsc::channel();
    // Run airline in a new Arbiter
    let airline_receiver_arbiter = Arbiter::new();
    let airline_rec_logger = logger_address.clone();
    let airline_rec_stream = airline_stream.try_clone();
    let airline_receiver_execution = async move {
        let airline_receiver_address =
            ExternalEntityReceiver::new("AIRLINE", airline_rec_stream, airline_rec_logger).start();
        airline_rec_s
            .send(airline_receiver_address)
            .expect("problema al enviar address!");
    };
    airline_receiver_arbiter.spawn(airline_receiver_execution);
    let airline_receiver_address = airline_rec_rec.recv().unwrap();

    // ********** Run Bank Receiver **********
    let (bank_rec_s, bank_rec_rec) = mpsc::channel();
    // Run airline in a new Arbiter
    let bank_rec_logger = logger_address.clone();
    let bank_receiver_arbiter = Arbiter::new();
    let bank_rec_stream = bank_stream.try_clone();
    let bank_receiver_execution = async move {
        let bank_receiver_address =
            ExternalEntityReceiver::new("BANK", bank_rec_stream, bank_rec_logger).start();
        bank_rec_s
            .send(bank_receiver_address)
            .expect("problema al enviar address!");
    };
    bank_receiver_arbiter.spawn(bank_receiver_execution);
    let bank_receiver_address = bank_rec_rec.recv().unwrap();


    //********** Run Hotel Receiver **********
    let (hotel_rec_s, hotel_rec_rec) = mpsc::channel();
    let hotel_receiver_arbiter = Arbiter::new();
    let hotel_rec_logger = logger_address.clone();
    let hotel_rec_stream = hotel_stream.try_clone();
    let hotel_receiver_execution = async move {
        let hotel_receiver_address =
            ExternalEntityReceiver::new("HOTEL", hotel_rec_stream, hotel_rec_logger)
                .start();
        hotel_rec_s
            .send(hotel_receiver_address)
            .expect("problema al enviar address!");
    };
    hotel_receiver_arbiter.spawn(hotel_receiver_execution);
    let hotel_receiver_address = hotel_rec_rec.recv().unwrap();


    //********** Run Airline Sender **********
    let (airline_sen_s, airline_sen_rec) = mpsc::channel();
    let arbiter = Arbiter::new();
    let airline_send_logger = logger_address.clone();
    let airline_send_stream = airline_stream.try_clone();
    let execution = async move {
        let address = ExternalEntitySender::new("AIRLINE", airline_send_stream, airline_send_logger, airline_receiver_address)
            .start();
        airline_sen_s
            .send(address)
            .expect("airline send, problema al enviar!");
    };
    arbiter.spawn(execution);
    let airline_sender_address = airline_sen_rec.recv().unwrap();

    //********** Run Bank Sender **********
    let (bank_sen_s, bank_sen_rec) = mpsc::channel();
    let bank_sender_arbiter = Arbiter::new();
    let bank_send_logger = logger_address.clone();
    let bank_send_stream = bank_stream.try_clone();
    let bank_sender_execution = async move {
        let bank_sender_address = ExternalEntitySender::new("BANK", bank_send_stream, bank_send_logger, bank_receiver_address)
            .start();
        bank_sen_s
            .send(bank_sender_address)
            .expect("bank send, problema al enviar!");
    };
    bank_sender_arbiter.spawn(bank_sender_execution);
    let bank_sender_address = bank_sen_rec.recv().unwrap();

    //********** Run Hotel Sender **********
    let (hotel_sen_s, hotel_sen_rec) = mpsc::channel();
    let hotel_sender_arbiter = Arbiter::new();
    let hotel_send_logger = logger_address.clone();
    let hotel_send_stream = hotel_stream.try_clone();
    let hotel_sender_execution = async move {
        let hotel_sender_address = ExternalEntitySender::new("HOTEL", hotel_send_stream, hotel_send_logger, hotel_receiver_address)
            .start();
        hotel_sen_s
            .send(hotel_sender_address)
            .expect("hotel send, problema al enviar!");
    };
    hotel_sender_arbiter.spawn(hotel_sender_execution);
    let hotel_sender_address = hotel_sen_rec.recv().unwrap();

    //********** Run Writer **********
    let (writer_s, writer_rec) = mpsc::channel();
    let writer_arbiter = Arbiter::new();
    let writer_logger = logger_address.clone();
    let writer_execution = async move {
        let writer_address =
            Writer::new("failed-transactions", writer_logger)
                .start();
        writer_s
            .send(writer_address)
            .expect("writer send, problema al enviar!");
    };
    arbiter.spawn(writer_execution);
    let writer_address = writer_rec.recv().unwrap();


    //********** Run Transaction Manager **********
    let (tx, rx) = mpsc::channel();
    let tm_arbiter = Arbiter::new();
    let tm_logger  = logger_address.clone();
    let tm_execution = async move {
        let tm_address = TransactionManager::new(
            airline_sender_address.clone(),
            bank_sender_address.clone(),
            hotel_sender_address.clone(),
            tm_logger,
            writer_address.clone(),
        )
            .start();
        tx.send(tm_address)
            .expect("pp send, problema al enviar!");
    };
    tm_arbiter.spawn(tm_execution);
    let tm_address = rx.recv().unwrap();

    //********** Run Reader **********
    let arbiter = Arbiter::new();
    let execution = async move {
        let address = Reader::new("archivo.csv", tm_address, logger_address.clone()).start();
        address.do_send(ParseTouristPackage());
    };
    arbiter.spawn(execution);

    arbiter.join();
    logger_arbiter.join();

    Ok(())
}










    /*
     * let airline_receiver = run_receiver("AIRLINE", airline_stream.as_ref(), logger_address.clone());
     * let bank_receiver = run_receiver("BANK", bank_stream.as_ref(), logger_address.clone());
     * let hotel_receiver = run_receiver("HOTEL", hotel_stream.as_ref(), logger_address.clone());
     *
     * let airline_sender = run_sender("AIRLINE", airline_stream.as_ref(), logger_address.clone(), airline_receiver);
     * let bank_sender = run_sender("BANK", bank_stream.as_ref(), logger_address.clone(), bank_receiver);
     * let hotel_sender = run_sender("HOTEL", hotel_stream.as_ref(), logger_address.clone(), hotel_receiver);
     */


    /*
     * init_app();
     *  let argv = args().collect::<Vec<String>>();
     *  if argv.len() != 2 {
     *      eprintln!("Uso: ./al-globo <configuracion.json>");
     *      exit(1);
     *  }
     *
     *  System::new().block_on(async {
     *      match Configuration::new(&argv[1]) {
     *          Ok(config) => init_app(config),
     *          Err(e) => {
     *              eprintln!("Error parseando configuración {:?}", e.to_string());
     *              exit(1);
     *          }
     *      };
     *      // This made it worked
     *      tokio::signal::ctrl_c().await.unwrap();
     *      println!("Ctrl-C received, shutting down");
     *      System::current().stop();
     *  });
     */
//}
