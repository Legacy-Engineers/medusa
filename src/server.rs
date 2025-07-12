// src/server.rs
use crate::client_handler::handle_client;
use crate::store::Store;
use std::net::TcpListener;
use std::thread;

pub fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:2312").expect("Could not bind port");
    println!("Medusa server running on 127.0.0.1:2312");

    // Create a shared store instance
    let store = Store::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_clone = store.clone();
                thread::spawn(move || {
                    handle_client(stream, store_clone);
                });
            }
            Err(e) => {
                eprintln!("Failed connection: {}", e);
            }
        }
    }
}
