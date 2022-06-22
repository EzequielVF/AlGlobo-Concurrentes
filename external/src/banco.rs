mod server;
mod logger;

use crate::server::run;

const IP: &str = "127.0.0.1";
const PORT: &str = "3001";
const SERVICE_NAME: &str = "banco";

fn main() {
    run(IP,PORT, SERVICE_NAME);
}