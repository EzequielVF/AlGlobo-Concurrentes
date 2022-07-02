use std::process::exit;

use std::env::args;

mod config;
mod writer;
mod reader;
mod types;
mod logger;
mod transaction_manager;
mod external_entity_sender;
mod communication;
mod external_entity_receiver;

use actix::prelude::*;
use crate::logger::Logger;
use crate::transaction_manager::TransactionManager;
use crate::reader::Reader;
use crate::writer::Writer;

fn main() {
    // let argv = args().collect::<Vec<String>>();
    // if argv.len() != 2 {
    //     eprintln!("Uso: ./al-globo <configuracion.json>");
    //     exit(1);
    // }
    //
    // match Configuration::new(&argv[1]) {
    //     Ok(config) => {
    //         // Aca está la config
    //     }
    //     Err(e) => {
    //         eprintln!("Error parseando configuración {:?}", e.to_string());
    //         exit(1);
    //     }
    // };
    // let addr_logger:Addr<Logger>= SyncArbiter::start(1, || Logger);
    //
    // let addr_writer:Addr<Writer> = SyncArbiter::start(1, |addr_logger| Writer);
    //
    // let addr_airline: Addr<ExternalEntity> = SyncArbiter::start(2, |"AIRLINE", "127.0.0.1", "3000",
    //                                           addr_pp,addr_logger| ExternalEntity);
    //
    // let addr_bank: Addr<ExternalEntity> = SyncArbiter::start(2,|"BANK", "127.0.0.1", "3001",
    //                                       addr_pp,addr_logger| ExternalEntity);
    //
    // let addr_hotel: Addr<ExternalEntity> = SyncArbiter::start(2,|"HOTEL", "127.0.0.1", "3002",
    //                                        addr_pp,addr_logger| ExternalEntity);
    //
    // let addr_pp: Addr<TransactionManager>= SyncArbiter::start(1, |addr_airline, addr_bank, addr_hotel,
    //                                                               addr_writer, addr_logger| PaymentProcessor);
    //
    // let addr_reader: Addr<Reader> = SyncArbiter::start(1, |addr_pp,addr_logger| Reader);


}