use crate::server::run;

mod server;
mod logger;

const IP: &str = "127.0.0.1";
const PORT: &str = "3000";
const SERVICE_NAME: &str = "aerolinea";


fn main() {
    run(IP,PORT, SERVICE_NAME);
}