mod server;
mod logger;

const IP: &str = "127.0.0.1";
const PORT: &str = "3000";
const SERVICE_NAME: &str = "aerolinea";

use crate::server::Server;

fn main() {
    let server = Server::new(IP,PORT, SERVICE_NAME);
    server.run();
}