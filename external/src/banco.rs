mod comunicacion;

use crate::comunicacion::run;

const IP: &str = "127.0.0.1";
const PORT: &str = "3001";
const SERVICE_NAME: &str = "BANCO";

fn main() {
    run(IP,PORT);
}