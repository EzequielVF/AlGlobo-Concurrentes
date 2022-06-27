use crate::server::run;

mod logger;
mod server;

const IP: &str = "127.0.0.1";
const PORT: &str = "3000";
const SERVICE_NAME: &str = "aerolinea";

fn main() {
    match run(IP, PORT, SERVICE_NAME) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Error ejecutando {}", SERVICE_NAME)
        }
    };
}
