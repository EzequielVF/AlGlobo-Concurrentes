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

const LOG_FILE: &str = "alglobo/src/archivo.csv";

#[actix_rt::main]
async fn main() {

    //  ********** Run Logger **********

    //Create channel to comunicate with logger
    let (logger_s, logger_rec) = mpsc::channel();

    // Run logger in a new Arbiter
    let logger_arbiter = Arbiter::new();
    let logger_execution = async move {
        let logger_address = Logger::new(LOG_FILE).start();
        logger_s.send(logger_address);
    };
    logger_arbiter.spawn(logger_execution);
    let logger_address = logger_rec.recv().unwrap();


    // ********** Run Payment Processor *************+
    logger_address.try_send(Log("Iniciando Procesador de Pagos".to_string()));

    let (pp_s, pp_rec) = mpsc::channel();
    let pp_arbiter = Arbiter::new();
    let pp_logger_address = logger_address.clone();
    let pp_execution = async move {
        let bank_address = ExternalEntity::new("BANK", IP, PORT_BANCO).start();
        let airline_address = ExternalEntity::new("AIRLINE", IP, PORT_AEROLINEA).start();
        let hotel_address = ExternalEntity::new("HOTEL", IP, PORT_HOTEL).start();
        let pp_addr = PaymentProcessor::new(bank_address, airline_address, hotel_address, pp_logger_address).start();
        pp_s.send(pp_addr);
    };
    pp_arbiter.spawn(pp_execution);
    let pp_addr = pp_rec.recv().unwrap();


    // ********** Run Package Reader *************
    logger_address.try_send(Log("Iniciando Package Reader".to_string()));
    let reader_arbiter = Arbiter::new();
    let reader_execution = async move {
        let reader_addr = Reader::new(LOG_FILE, pp_addr).start();
        reader_addr.try_send(LeerPaquete());
    };
    reader_arbiter.spawn(reader_execution);
}

