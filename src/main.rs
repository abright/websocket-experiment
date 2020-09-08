use std::{
    io::Error as IoError,
    net::TcpListener,
    sync::atomic::{AtomicUsize, Ordering},
    thread::Builder,
};
use thiserror::Error;
use tungstenite::server::accept;

#[derive(Debug, Error)]
enum ServerError {
    #[error("could not bind socket")]
    Bind(#[source] IoError),
    #[error("unable to spawn worker thread")]
    SpawnThread(#[source] IoError),
    #[error("error accepting connection")]
    IncomingStream(#[source] IoError),
}

fn next_worker_id() -> usize {
    static WORKER_ID: AtomicUsize = AtomicUsize::new(0);
    WORKER_ID.fetch_add(1, Ordering::SeqCst)
}

fn run_server() -> Result<(), ServerError> {
    // connect to ws://127.0.0.1:9002
    let server = TcpListener::bind("127.0.0.1:9002").map_err(ServerError::Bind)?;
    for stream in server.incoming() {
        let stream = stream.map_err(ServerError::IncomingStream)?;
        let worker_id = next_worker_id();
        Builder::new()
            .name(format!("worker-{}", worker_id))
            .spawn(move || {
                let mut websocket = accept(stream).expect("can't accept connection");
                loop {
                    let msg = websocket.read_message().expect("can't read message");

                    // don't send back ping/pong messages
                    if msg.is_binary() || msg.is_text() {
                        websocket.write_message(msg).expect("can't write message");
                    }
                }
            })
            .map_err(ServerError::SpawnThread)?;
    }
    Ok(())
}

fn main() {
    run_server().unwrap();
}
