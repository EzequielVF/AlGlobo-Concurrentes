use crate::communication::connect_to_server;
use actix::prelude::*;
use std::collections::HashMap;
use std::env::args;
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

fn init_app(configuration: Configuration) -> Result<(), &'static str> {
    /*let config {
    log_file,
    payment_transactions,
    failed_transactions,
    .. } = configuration;*/

    let addr_logger = SyncArbiter::start(1, move || Logger::new(&(configuration.log_file)));

    let writer_addr = SyncArbiter::start(1, move || {
        Writer::new(&(configuration.failed_transactions), addr_logger.clone())
    });

    // conexion aerolinea
    let airline_stream = connect_to_server("127.0.0.1", "3000")?;

    let airline_receiver_addr = SyncArbiter::start(1, || {
        ExternalEntityReceiver::new("AIRLINE", airline_stream.try_clone(), addr_logger.clone())
    });

    let airline_sender_addr = SyncArbiter::start(2, || {
        ExternalEntitySender::new(
            "AIRLINE",
            airline_stream.try_clone(),
            addr_logger.clone(),
            airline_receiver_addr.clone(),
        )
    });

    // Conexion banco
    let bank_stream = connect_to_server("127.0.0.1", "3001")?;

    let bank_receiver_addr = SyncArbiter::start(1, || {
        ExternalEntityReceiver::new("BANK", bank_stream.try_clone(), addr_logger.clone())
    });

    let bank_sender_addr = SyncArbiter::start(2, || {
        ExternalEntitySender::new(
            "BANK",
            bank_stream.try_clone(),
            addr_logger.clone(),
            bank_receiver_addr.clone(),
        )
    });

    // Conexion Hotel
    let hotel_stream = connect_to_server("127.0.0.1", "3002")?;

    let hotel_receiver_addr = SyncArbiter::start(1, || {
        ExternalEntityReceiver::new("HOTEL", hotel_stream.try_clone(), addr_logger.clone())
    });

    let hotel_sender_addr = SyncArbiter::start(2, || {
        ExternalEntitySender::new(
            "HOTEL",
            hotel_stream.try_clone(),
            addr_logger.clone(),
            hotel_receiver_addr.clone(),
        )
    });

    let transaction_manager_addr = SyncArbiter::start(1, || {
        TransactionManager::new(
            airline_sender_addr.clone(),
            bank_sender_addr.clone(),
            hotel_sender_addr.clone(),
            addr_logger.clone(),
            writer_addr.clone(),
        )
    });

    let addr_reader: Addr<Reader> = SyncArbiter::start(1, || {
        Reader::new(
            &(configuration.payment_transactions),
            transaction_manager_addr.clone(),
            addr_logger.clone(),
        )
    });

    addr_reader.do_send(ParseTouristPackage());
    Ok(())
}

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        eprintln!("Uso: ./al-globo <configuracion.json>");
        exit(1);
    }

    System::new().block_on(async {
        match Configuration::new(&argv[1]) {
            Ok(config) => init_app(config),
            Err(e) => {
                eprintln!("Error parseando configuración {:?}", e.to_string());
                exit(1);
            }
        };
        // This made it worked
        tokio::signal::ctrl_c().await.unwrap();
        println!("Ctrl-C received, shutting down");
        System::current().stop();
    });
}
