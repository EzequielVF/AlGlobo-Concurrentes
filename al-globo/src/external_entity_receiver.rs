use actix::prelude::*;
use std::net::TcpStream;
use actix::{Actor, Addr, Context, Handler, SyncContext, Message};
use crate::communication::{connect_to_server, read_answer, send_transaction_result};
use crate::{Logger, TransactionManager};
use crate::logger::Log;
use crate::transaction_manager::ProcessEntityAnswer;
use crate::types::{Answer, EntityAnswer, ServerResponse};

pub struct ExternalEntityReceiver {
    name: String,
    stream: Result<TcpStream, std::io::Error>,
    logger_addr: Addr<Logger>,
}

impl Actor for ExternalEntityReceiver{
    type Context = SyncContext<Self>;
}

impl ExternalEntityReceiver {
    pub fn new(name: &str, ip: &str, port: &str, logger_addr: Addr<Logger>) -> Self {
        let channel = connect_to_server(ip, port);
        ExternalEntityReceiver {
            name: name.to_string(),
            stream: channel,
            logger_addr,
        }
    }
}

/// Encargado de leer respuestas de servidores externos
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ReceiveServerAnswer(
    pub Addr<TransactionManager>
);

impl Handler<ReceiveServerAnswer> for ExternalEntityReceiver {
    type Result = ();

    fn handle(&mut self, msg: ReceiveServerAnswer, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction_manager_addr = msg.0;

        match &self.stream {
            Ok(tcpStream) => {
                let server_response: ServerResponse = read_answer(&mut tcpStream.try_clone().unwrap());

                let mut entity_answer = EntityAnswer {
                    entity_name: self.name.clone(),
                    transaction_id: server_response.transaction_id,
                    answer: Answer::Ok
                };

                if server_response.response == false {
                    entity_answer.answer = Answer::Failed;
                }

                transaction_manager_addr.do_send(ProcessEntityAnswer(entity_answer));
            }
            Err(_) => {
                self.logger_addr.do_send(Log(format!("Ocurrió un error al intentar leer respuesta del servidor [{}]", self.name)));
            }
        }




    }
}
