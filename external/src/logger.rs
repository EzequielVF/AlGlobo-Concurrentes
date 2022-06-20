use std::fmt::format;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{Duration, SystemTime};

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(name: String) -> Self {
        let filename = format!("{}.log", name);

        let log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .expect("Error creando archivo de log");

        Logger {
            file: log_file
        }
    }

    pub fn log(&mut self, detalle: &str) {
        let tiempo = "2022;07;01";
        let mensaje = format!("[{}] - {}\n", tiempo, detalle);

        self.file.write_all(mensaje.as_bytes())
            .expect("Error escribiendo archivo de log");
    }
}


