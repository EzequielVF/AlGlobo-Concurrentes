mod server;
mod logger;

use crate::server::Server;

const IP: &str = "127.0.0.1";
const PORT: &str = "3002";
const SERVICE_NAME: &str = "hotel";

fn main() {
    let server = Server::new(IP,PORT, SERVICE_NAME);
    server.run();
}