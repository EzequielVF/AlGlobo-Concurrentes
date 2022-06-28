use std::fs::{File, OpenOptions};
use std::io::Write;

use actix::{Actor, Context, Handler, Message};
use chrono::Local;

pub struct Logger {
    file: File,
}

impl Actor for Logger {
    type Context = Context<Self>;
}

// Log para la escritura de mensajes informativos y de error
impl Logger {
    pub fn new(name: &str) -> Self {
        let filename = format!("{}.log", name);

        let log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .expect("Error creando archivo de log");

        Logger { file: log_file }
    }
}

// Mensaje para realizar escritura en el _log_
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Log(pub String);

impl Handler<Log> for Logger {
    type Result = ();

    fn handle(&mut self, msg: Log, _ctx: &mut Context<Self>) -> Self::Result {
        let time = Local::now().to_string();
        let message = format!("[{}] - {}\n", time, msg.0);
        self.file
            .write_all(message.as_bytes())
            .expect("Error escribiendo archivo de log");
    }
}
