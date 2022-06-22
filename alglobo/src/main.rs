mod comunicacion;
mod client;
mod entity_actor;
mod procesador_pagos;

use crate::client::run;

const CLIENT_NAME: &str = "PROCESADOR_PAGOS";

fn main() {
    run();
}