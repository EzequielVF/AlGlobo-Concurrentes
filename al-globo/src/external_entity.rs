use std::net::TcpStream;

use actix::{Actor, Addr, Context, Handler, Message, SyncContext};

use crate::communication::{connect_to_server, read_answer, send_package, send_transaction_result};
use crate::payment_processor::{EntityAnswer, PaymentProcessor};
use crate::{Log, Logger};
use crate::logger::Log;
use crate::types::{EntityAnswer, Transaction};

/// Representación de la Entidad Externa (*Aerolínea*, *Banco* u *Hotel*)
/// a la cual el *Procesador de Pagos* se conecta para enviar las transacciones
pub struct ExternalEntity {
    name: String,
    stream: Result<TcpStream, std::io::Error>,
    logger_address: Addr<Logger>,
}

impl Actor for ExternalEntity {
    type Context = SyncContext<Self>;
}

impl ExternalEntity {
    pub fn new(name: &str, ip: &str, port: &str, logger_address: Addr<Logger>) -> Self {
        let channel = connect_to_server(ip, port);
        ExternalEntity {
            name: name.to_string(),
            stream: channel,
            logger_address,
        }
    }
}

/// Mensaje de nuevo Pago entrante a enviar hacia los servicios externos
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTransaction(pub Transaction, pub Addr<PaymentProcessor>);

impl Handler<SendTransaction> for ExternalEntity {
    type Result = ();

    fn handle(&mut self, msg: SendTransaction, _ctx: &mut Context<Self>) -> Self::Result {
        let transaction = msg.0;
        let addr = msg.1;
        match &self.stream {
            Ok(t) => {
                let mut channel = t.try_clone().unwrap();
                send_package(&mut channel, transaction.clone(), &self.name);

                self.logger_address.do_send(Log(format!(
                    "[{}], Envío paquete de código {} para procesamiento",
                    self.name, transaction.id
                )));

                let response = read_answer(&mut t.try_clone().unwrap());
                self.logger_address.do_send(Log(format!(
                    "[{}], Recibo respuesta paquete de código {} por parte del servidor",
                    self.name, transaction.id
                )));

                addr.do_send(EntityAnswer(self.name.clone(), transaction.id, response))
            }
            Err(_) => {}
        }
    }
}

/// Representación de la respuesta del pago enviado
/// por parte de la Entidad Externa (*Aerolínea*, *Banco* u *Hotel*)
#[derive(Clone, Debug)]
pub struct TransactionResult {
    pub transaction_id: String,
    pub result: bool,
}

/// Mensaje de resultado de transacción recibido desde los servicios externos
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct TransactionResultMessage(pub TransactionResult);

impl Handler<TransactionResultMessage> for ExternalEntity {
    type Result = ();

    fn handle(&mut self, msg: TransactionResultMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let transaction_result = msg.0;
        let parsed_transaction_result = if transaction_result.result {
            "confirmación"
        } else {
            "rollback"
        };

        self.logger_address.do_send(Log(format!(
            "[{}] Se envía {} para transacción {}",
            self.name, parsed_transaction_result, transaction_result.transaction_id
        )));

        match &self.stream {
            Ok(t) => {
                let mut channel = t.try_clone().unwrap();
                send_transaction_result(&mut channel, transaction_result);
            }
            Err(_) => {}
        }
    }
}
