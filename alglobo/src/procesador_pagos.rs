mod comunicacion;
mod client_model;

use crate::client_model::run;

const IP: &str = "127.0.0.1";
const CLIENT_NAME: &str = "PROCESADOR_PAGOS";

fn main() {
    run(IP);
}