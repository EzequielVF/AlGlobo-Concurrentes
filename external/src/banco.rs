mod server;
mod logger;

use crate::server::Server;

const IP: &str = "127.0.0.1";
const PORT: &str = "3001";
const SERVICE_NAME: &str = "banco";

fn main() {
    let server = Server::new(IP,PORT, SERVICE_NAME);
    server.run();
}