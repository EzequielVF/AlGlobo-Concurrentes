mod client_model;
mod comunicacion;

use crate::client_model::run;

const IP: &str = "127.0.0.1";
const PORT: &str = "3003";
const CLIENT_NAME: &str = "PROCESADOR_PAGOS";

fn main() {
    run(IP,PORT);
}