use actix::prelude::*;
use std::net::TcpStream;

use actix::{Actor, Addr, Context, Handler, Message, SyncContext};

use crate::communication::{connect_to_server, read_answer, send_package, send_transaction_result};
use crate::transaction_manager::{TransactionManager};
use crate::{Logger};
use crate::external_entity_receiver::{ExternalEntityReceiver, ReceiveServerAnswer};
use crate::logger::Log;
use crate::types::{EntityAnswer, Transaction, TransactionResult};

/// Representación de la Entidad Externa (*Aerolínea*, *Banco* u *Hotel*)
/// a la cual el *Procesador de Pagos* se conecta para enviar las transacciones
pub struct ExternalEntitySender {
    name: String,
    stream: Result<TcpStream, std::io::Error>,
    logger_addr: Addr<Logger>,
    external_entity_receiver_addr :Addr<ExternalEntityReceiver>
}

impl Actor for ExternalEntitySender {
    type Context = SyncContext<Self>;
}

impl ExternalEntitySender {
    pub fn new(name: &str, ip: &str, port: &str, logger_addr: Addr<Logger>,
               external_entity_receiver_addr: Addr<ExternalEntityReceiver>) -> Self {
        let channel = connect_to_server(ip, port);
        ExternalEntitySender {
            name: name.to_string(),
            stream: channel,
            logger_addr,
            external_entity_receiver_addr
        }
    }
}

/// Mensaje de nuevo Pago entrante a enviar hacia los servicios extern
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTransactionToServer(pub Transaction, pub Addr<TransactionManager>);

impl Handler<SendTransactionToServer> for ExternalEntitySender {
    type Result = ();

    fn handle(&mut self, msg: SendTransactionToServer, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let sender = msg.1;
        let transaction = msg.0;
        
        match &self.stream {
            Ok(tcpStream) => {
                let mut channel = tcpStream.try_clone().unwrap();
                self.logger_addr.do_send(Log(format!("Se envía transacción de id {} al servidor [{}]", transaction.id, self.name)));
                send_package(&mut channel, transaction.clone(), &self.name);
                self.external_entity_receiver_addr.do_send(ReceiveServerAnswer(sender));
            }
            Err(_) => {
                self.logger_addr.do_send(Log(format!("Ocurrió un error al intentar enviar transacción de id {} al servidor [{}]", transaction.id, self.name)));
            }
        }
    }
}


/// Mensaje de resultado de transacción recibido desde los servicios externos
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct SendConfirmationOrRollbackToServer(pub TransactionResult);

impl Handler<SendConfirmationOrRollbackToServer> for ExternalEntitySender {
    type Result = ();

    fn handle(&mut self, msg: SendConfirmationOrRollbackToServer, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction_result = msg.0;

        let mut message = "confirmacion";

        if !transaction_result.success {
            message = "rollback";
        }

        match &self.stream {
            Ok(t) => {
                let mut channel = t.try_clone().unwrap();
                send_transaction_result(&mut channel, transaction_result.clone());
                self.logger_addr.do_send(Log(format!(
                    "Se envía mensage {} para transacción {} al servidor [{}]",
                    message, transaction_result.transaction_id, self.name
                )));
            }
            Err(_) => {
                self.logger_addr.do_send(Log(format!("Ocurrió un error al intentar enviar mensaje  {} para transacción {} al servidor [{}]", message, transaction_result.transaction_id, self.name)));
            }
        }
    }
}
