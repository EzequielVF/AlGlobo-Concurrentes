mod server_model;
mod comunicacion;

const IP: &str = "127.0.0.1";
const PORT: &str = "3000";
const SERVICE_NAME: &str = "AEROLINEA";

use crate::server_model::run;

fn main() {
    run(IP,PORT);
}