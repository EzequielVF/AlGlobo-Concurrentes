use actix::{Actor, Addr, Context, Handler, Message, ResponseActFuture, System};
use actix_rt::Arbiter;
use crate::client::PaqueteTuristico;
use crate::entity_actor::{EntityActor, ProcesarPaquete};
use crate::entity_actor::ProcesarPaquete as OtherProcesarPaquete;

const PORT_AEROLINEA: &str = "3000";
const PORT_BANCO: &str = "3001";
const PORT_HOTEL: &str = "3002";
const FILE: &str = "alglobo/src/archivo.csv";

pub struct ProcesadorPagos {
    addr_bank: Addr<EntityActor>,
    addr_airline: Addr<EntityActor>,
    addr_hotel: Addr<EntityActor>
}

impl Actor for ProcesadorPagos {
    type Context = Context<Self>;
}

impl ProcesadorPagos {
    pub fn new(ip: &str) -> Self {
        let banco_addr = crate::entity_actor::EntityActor::new(ip, PORT_BANCO, String::from(FILE)).start();
        let aerolinea_addr = crate::entity_actor::EntityActor::new(ip, PORT_AEROLINEA, String::from(FILE)).start();
        let hotel_addr = crate::entity_actor::EntityActor::new(ip, PORT_HOTEL, String::from(FILE)).start();
        ProcesadorPagos {
            addr_bank: banco_addr,
            addr_airline: aerolinea_addr,
            addr_hotel: hotel_addr
        }
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Procesar(pub PaqueteTuristico);

impl Handler<Procesar> for ProcesadorPagos {
    type Result = bool;

    fn handle(&mut self, msg: Procesar, _ctx: &mut Context<Self>) -> Self::Result {
        self.addr_hotel.try_send(ProcesarPaquete(msg.0.clone()));
        self.addr_airline.try_send(ProcesarPaquete(msg.0.clone()));
        self.addr_bank.try_send(ProcesarPaquete(msg.0.clone()));
        true
    }
}