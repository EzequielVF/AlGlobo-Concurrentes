use actix::{Actor,  Handler, Message, SyncContext};
use chrono::{Local};
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive()]
pub struct Logger {
    pub file: File,
}

impl Actor for Logger {
    type Context = SyncContext<Self>;
}

impl Logger {
    pub fn new(path:&str) -> Self {
        let log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect("Error creando archivo de log");
        Logger { file: log_file }
    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct Log(pub String);

impl Handler<Log> for Logger {
    type Result = ();
    fn handle(&mut self, msg: Log, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let message = msg.0;
        let time = Local::now().to_string();
        let log = format!("[{}] - {}\n", time, message);

        print!("{}", log);
        self.file.write_all(log.as_bytes()) .expect("Error escribiendo archivo de log");
    }
}
