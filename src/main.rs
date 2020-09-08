use std::{
    net::TcpListener,
    sync::atomic::{AtomicUsize, Ordering},
    thread::Builder,
};
use tungstenite::server::accept;

fn next_worker_id() -> usize {
    static WORKER_ID: AtomicUsize = AtomicUsize::new(0);
    WORKER_ID.fetch_add(1, Ordering::SeqCst)
}

fn main() {
    // connect to ws://127.0.0.1:9002
    let server = TcpListener::bind("127.0.0.1:9002").expect("couldn't create TCP listener");
    for stream in server.incoming() {
        let worker_id = next_worker_id();
        Builder::new()
            .name(format!("worker-{}", worker_id))
            .spawn(move || {
                let mut websocket = accept(stream.expect("invalid incoming request"))
                    .expect("can't accept connection");
                loop {
                    let msg = websocket.read_message().expect("can't read message");

                    // don't send back ping/pong messages
                    if msg.is_binary() || msg.is_text() {
                        websocket.write_message(msg).expect("can't write message");
                    }
                }
            })
            .expect("can't spawn worker thread");
    }
}
