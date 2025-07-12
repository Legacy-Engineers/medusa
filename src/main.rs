mod client_handler;
mod server;
mod store;

use server::start_server;

fn main() {
    println!("Starting Medusa server...");
    start_server();
}
