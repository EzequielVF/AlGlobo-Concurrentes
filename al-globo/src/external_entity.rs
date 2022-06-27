use std::net::TcpStream;

use actix::{Actor, Addr, Context, Handler, Message};

use crate::comunicacion::{
    conectar_con_servidor, enviar_paquete, enviar_resultado, leer_respuesta,
};
use crate::payment_processor::EntityAnswer;
use crate::{Log, Logger, PaqueteTuristico, PaymentProcessor};

pub struct ExternalEntity {
    name: String,
    stream: Result<TcpStream, std::io::Error>,
    logger_address: Addr<Logger>,
}

impl Actor for ExternalEntity {
    type Context = Context<Self>;
}

impl ExternalEntity {
    pub fn new(name: &str, ip: &str, port: &str, logger_address: Addr<Logger>) -> Self {
        let channel = conectar_con_servidor(ip, port);
        ExternalEntity {
            name: name.to_string(),
            stream: channel,
            logger_address,
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ExtEntNewPayment(pub PaqueteTuristico, pub Addr<PaymentProcessor>);

impl Handler<ExtEntNewPayment> for ExternalEntity {
    type Result = ();

    fn handle(&mut self, msg: ExtEntNewPayment, _ctx: &mut Context<Self>) -> Self::Result {
        match &self.stream {
            Ok(t) => {
                let mut channel = t.try_clone().unwrap();
                enviar_paquete(&mut channel, msg.0.clone(), &self.name);
                self.logger_address.do_send(Log(format!(
                    "[{}], Envio paquete de codigo {} para procesamiento",
                    self.name, msg.0.id
                )));
                let response = leer_respuesta(&mut t.try_clone().unwrap());
                self.logger_address.do_send(Log(format!(
                    "[{}], Recibo respuesta paquete de codigo {} por parte del servidor",
                    self.name, msg.0.id
                )));
                (msg.1).do_send(EntityAnswer(self.name.clone(), msg.0.id, response));
            }
            Err(_) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub result: bool,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct TransactionResultMessage(pub TransactionResult);

impl Handler<TransactionResultMessage> for ExternalEntity {
    type Result = ();

    fn handle(&mut self, msg: TransactionResultMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.0.result {
            self.logger_address.do_send(Log(format!(
                "[{}] Se envía confirmación para transaccion {}",
                self.name, msg.0.transaction_id
            )));
        } else {
            self.logger_address.do_send(Log(format!(
                "[{}]  Se envía rollback para transacción {}",
                self.name, msg.0.transaction_id
            )));
        }
        match &self.stream {
            Ok(t) => {
                let mut channel = t.try_clone().unwrap();
                enviar_resultado(&mut channel, msg.0);
            }
            Err(_) => {}
        }
    }
}