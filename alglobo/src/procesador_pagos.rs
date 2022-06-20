mod comunicacion;
mod client;

use crate::client::run;

const IP: &str = "127.0.0.1";
const CLIENT_NAME: &str = "PROCESADOR_PAGOS";

fn main() {
    run(IP);
}