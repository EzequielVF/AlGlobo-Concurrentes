use std::net::TcpStream;
use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use actix::clock::sleep;

use crate::{Log, Logger, PaqueteTuristico, PaymentProcessor, PP_NewPayment};
use crate::comunicacion::{conectar_con_servidor, enviar_paquete, leer_respuesta};
use crate::payment_processor::{EntityAnswer, RequestState};

pub struct ExternalEntity {
    name: String,
    stream: Result<TcpStream, std::io::Error>,
    logger_address: Addr<Logger>
}

impl Actor for ExternalEntity {
    type Context = Context<Self>;
}

impl ExternalEntity {
    pub fn new(name: &str, ip: &str, port: &str, logger_address:Addr<Logger>) -> Self {
        let channel = conectar_con_servidor(ip, port);
        ExternalEntity {
            name: name.to_string(),
            stream: channel,
            logger_address
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct EE_NewPayment(pub PaqueteTuristico, pub Addr<PaymentProcessor>);

impl Handler<EE_NewPayment> for ExternalEntity {
    type Result = ();

    fn handle(&mut self, msg: EE_NewPayment, _ctx: &mut Context<Self>) -> Self::Result {

        match &self.stream {
            Ok(t) => {
                let mut channel = t.try_clone().unwrap();
                enviar_paquete(&mut channel, msg.0.clone(), &self.name);
                self.logger_address.try_send(Log(format!("[{}], Envio paquete de codigo {} para procesamiento", self.name,msg.0.id)));
                let response = leer_respuesta(&mut t.try_clone().unwrap());
                self.logger_address.try_send(Log(format!("[{}], Recibo respuesta paquete de codigo {} por parte del servidor", self.name,msg.0.id)));
                (msg.1).try_send(EntityAnswer(self.name.clone(), msg.0.id.clone(), response));
            }
            Err(_) => {

            }
        }
    }
}

/*
pub struct RollBack();

impl Handler<RollBack> for ExternalEntity {
    type Result = ();

    fn handle(&mut self, msg: RollBack, _ctx: &mut Context<Self>) -> Self::Result {}
}
*/