use std::fs::{File, OpenOptions};
use std::io::Write;

use chrono::Local;

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(name: &str) -> Self {
        let filename = format!("logs/{}.log", name);

        let log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // si no queremos que borre sino que sume a lo que tiene lo remplazamos por .append(true)
            .open(filename)
            .expect("Error creando archivo de log");

        Logger { file: log_file }
    }

    pub fn log(&mut self, detalle: &str) {
        let tiempo = Local::now().to_string();
        let mensaje = format!("[{}] - {}\n", tiempo, detalle);

        self.file
            .write_all(mensaje.as_bytes())
            .expect("Error escribiendo archivo de log");
    }
}
