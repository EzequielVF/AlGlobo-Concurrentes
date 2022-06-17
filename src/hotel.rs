mod server_model;
mod comunicacion;

use crate::server_model::run;

const IP: &str = "127.0.0.1";
const PORT: &str = "3002";
const SERVICE_NAME: &str = "HOTEL";

fn main() {
    run(IP,PORT);
}