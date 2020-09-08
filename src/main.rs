use std::{net::TcpListener, thread::spawn};
use tungstenite::server::accept;

fn main() {
    // connect to ws://127.0.0.1:9002
    let server = TcpListener::bind("127.0.0.1:9002").expect("couldn't create TCP listener");
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.expect("invalid incoming request")).expect("can't accept connection");
            loop {
                let msg = websocket.read_message().expect("can't read message");
    
                // don't send back ping/pong messages
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg).expect("can't write message");
                }
            }
        });
    }
}