mod comunicacion;
mod client;
mod entity_actor;


use crate::client::run;

const CLIENT_NAME: &str = "PROCESADOR_PAGOS";

fn main() {
    run();
}