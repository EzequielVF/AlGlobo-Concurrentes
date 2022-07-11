use std::env;
use std::process::exit;

use crate::server::run;

mod logger;
mod server;

struct ServiceConfiguration {
    ip: String,
    port: String,
    service_name: String,
    success_rate: u8,
}

impl ServiceConfiguration {
    fn new(args: Vec<String>) -> ServiceConfiguration {
        const EXPECTED_ARGUMENTS: usize = 5;

        if args.len() != EXPECTED_ARGUMENTS {
            eprintln!("Argumentos insuficientes!");
            eprintln!("Uso: <ip> <puerto> <{{airline|bank|hotel}}> <tasa-exito>");
            exit(1);
        }

        let service_name: String;
        match args[3].as_str() {
            "airline" | "bank" | "hotel" => {
                service_name = String::from(&args[3]);
            }
            _ => {
                eprintln!("<nombre-servicio> debe ser {{airline|bank|hotel}}");
                exit(2);
            }
        }

        let success_rate: u8;
        match args[4].parse::<u8>() {
            Ok(rate) => {
                if rate > 100 {
                    eprintln!("<tasa-exito> debe estar entre 0 y 100");
                    exit(2);
                }
                success_rate = rate;
            }
            Err(e) => {
                eprintln!("Falló parseo de tasa-exito: {}", e);
                exit(2);
            }
        }

        ServiceConfiguration {
            ip: String::from(&args[1]),
            port: String::from(&args[2]),
            service_name,
            success_rate,
        }
    }
}

fn main() {
    let ServiceConfiguration {
        ip,
        port,
        service_name,
        success_rate
    } = ServiceConfiguration::new(env::args().collect());

    match run(&ip, &port, &service_name, success_rate) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Error ejecutando {}", service_name)
        }
    };
}
