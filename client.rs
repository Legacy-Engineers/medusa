use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:2312")?;
    println!("Connected to server");

    let read_stream = stream.try_clone()?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut reader = BufReader::new(read_stream);
        let mut buffer = String::new();

        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    print!("Server: {}", buffer);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    break;
                }
            }
        }
        let _ = tx.send(());
    });

    let stdin = io::stdin();
    println!("Type messages (or 'quit' to exit):");

    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                let message = format!("{}\n", input);
                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }

                if input.trim() == "quit" || input.trim() == "exit" {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    let _ = rx.recv();
    println!("Disconnected from server");
    Ok(())
}
