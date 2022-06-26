use std::sync::mpsc;
use std::thread;

use actix::Actor;
use actix_rt::{Arbiter, System};

use crate::comunicacion::Tipo;
use crate::comunicacion::Tipo::{Error, Pay, Succesfull, Unknown};
use crate::external_entity::ExternalEntity;
use crate::logger::{Log, Logger};
use crate::payment_processor::{PaqueteTuristico, PaymentProcessor, PP_NewPayment};
use crate::reader::{LeerPaquete, Reader};

mod external_entity;
mod payment_processor;
mod comunicacion;
mod logger;
mod reader;

const PORT_AEROLINEA: &str = "3000";
const PORT_BANCO: &str = "3001";
const PORT_HOTEL: &str = "3002";
const IP: &str = "127.0.0.1";

const LOG_FILE: &str = "logs/procesador-pagos";
const FILE: &str = "archivo.csv";

#[actix_rt::main]
async fn main() {
    //  ********** Run Logger **********
    // Create channel to comunicate with logger
    let (logger_s, logger_rec) = mpsc::channel();

    // Run logger in a new Arbiter
    let logger_arbiter = Arbiter::new();
    let logger_execution = async move {
        let logger_address = Logger::new(LOG_FILE).start();
        logger_s.send(logger_address);
    };
    logger_arbiter.spawn(logger_execution);
    let logger_address = logger_rec.recv().unwrap();

    // ********** Run Airline **********
    let (airline_s, airline_rec) = mpsc::channel();
    // Run airline in a new Arbiter
    let airline_logger_address = logger_address.clone();
    let airline_arbiter = Arbiter::new();
    let airline_execution = async move {
        let airline_address = ExternalEntity::new("AIRLINE", IP, PORT_AEROLINEA, airline_logger_address).start();
        airline_s.send(airline_address);
    };
    airline_arbiter.spawn(airline_execution);

    // ********** Run Bank **********
    let (bank_s, bank_rec) = mpsc::channel();
    let bank_logger_address = logger_address.clone();
    let bank_arbiter = Arbiter::new();
    let bank_execution = async move {
        let bank_address = ExternalEntity::new("BANK", IP, PORT_BANCO, bank_logger_address).start();
        bank_s.send(bank_address);
    };
    bank_arbiter.spawn(bank_execution);

    // ********** Run Hotel **********
    let (hotel_s, hotel_rec) = mpsc::channel();
    let hotel_logger_address = logger_address.clone();
    let hotel_arbiter = Arbiter::new();
    let hotel_execution = async move {
        let hotel_address = ExternalEntity::new("HOTEL", IP, PORT_HOTEL, hotel_logger_address).start();
        hotel_s.send(hotel_address);
    };
    hotel_arbiter.spawn(hotel_execution);

    // ********** Run Payment Processor *************+
    logger_address.try_send(Log("Iniciando Procesador de Pagos".to_string()));

    let (pp_tx, pp_rx) = mpsc::channel();
    let pp_arbiter = Arbiter::new();
    let pp_logger_address = logger_address.clone();
    let pp_execution = async move {
        let airline_address = airline_rec.recv().expect("Falló recepción dirección Aerolínea");
        let bank_address = bank_rec.recv().expect("Falló recepción dirección Banco");
        let hotel_address = hotel_rec.recv().expect("Falló recepción dirección Hotel");
        let pp_addr = PaymentProcessor::new(bank_address, airline_address, hotel_address, pp_logger_address).start();
        pp_tx.send(pp_addr);
    };
    pp_arbiter.spawn(pp_execution);
    let pp_addr = pp_rx.recv().unwrap();


    // ********** Run Package Reader *************
    logger_address.try_send(Log("Iniciando Package Reader".to_string()));
    let reader_arbiter = Arbiter::new();
    let reader_logger_address = logger_address.clone();
    let reader_execution = async move {
        let reader_addr = Reader::new(FILE, pp_addr, reader_logger_address).start();
        reader_addr.try_send(LeerPaquete());
    };
    reader_arbiter.spawn(reader_execution);

    pp_arbiter.join();
    reader_arbiter.join();
}

