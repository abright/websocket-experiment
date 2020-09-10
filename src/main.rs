use native_tls::{Error as TlsError, Identity, TlsAcceptor, TlsStream};
use std::{
    fs::File,
    io::{Error as IoError, Read},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread::Builder,
};
use thiserror::Error;
use tungstenite::{
    error::Error as WebSocketError, handshake::server::NoCallback, server::accept, HandshakeError,
    ServerHandshake,
};

#[derive(Debug, Error)]
enum ServerError {
    #[error("could not open certificate file")]
    OpenCertificate(#[source] IoError),
    #[error("could not read certificate file")]
    ReadCertificate(#[source] IoError),
    #[error("unable to create TLS identity from certificate")]
    CreateTLSIdentity(#[source] TlsError),
    #[error("could not bind socket")]
    Bind(#[source] IoError),
    #[error("could not create TLS acceptor")]
    CreateTLSAcceptor(#[source] TlsError),
    #[error("unable to spawn worker thread")]
    SpawnThread(#[source] IoError),
    #[error("error accepting connection")]
    IncomingStream(#[source] IoError),
}

#[derive(Debug, Error)]
// TODO: add some identifier to these errors (a thread id or port number maybe?)
enum WorkerError {
    #[error("unable to accept connection as WebSocket")]
    WebSocketAccept(#[source] HandshakeError<ServerHandshake<TlsStream<TcpStream>, NoCallback>>),
    #[error("couldn't read WebSocket message")]
    ReadMessage(#[source] WebSocketError),
    #[error("couldn't write WebSocket message")]
    WriteMessage(#[source] WebSocketError),
}

fn next_worker_id() -> usize {
    static WORKER_ID: AtomicUsize = AtomicUsize::new(0);
    WORKER_ID.fetch_add(1, Ordering::SeqCst)
}

fn handle_client(id: usize, stream: TlsStream<TcpStream>) -> Result<(), WorkerError> {
    let mut websocket = accept(stream).map_err(WorkerError::WebSocketAccept)?;
    loop {
        let msg = match websocket.read_message() {
            Err(WebSocketError::ConnectionClosed) => {
                println!("client disconnected from worker {}", id);
                break;
            }
            a => a.map_err(WorkerError::ReadMessage),
        }?;

        // don't send back ping/pong messages
        if msg.is_binary() || msg.is_text() {
            websocket
                .write_message(msg)
                .map_err(WorkerError::WriteMessage)?;
        }
    }
    Ok(())
}

fn run_server() -> Result<(), ServerError> {
    let mut file = File::open("test.pfx").map_err(ServerError::OpenCertificate)?;
    let mut identity = vec![];
    file.read_to_end(&mut identity)
        .map_err(ServerError::ReadCertificate)?;
    let identity =
        Identity::from_pkcs12(&identity, "test").map_err(ServerError::CreateTLSIdentity)?;

    // connect to wss://localhost:9002
    let server = TcpListener::bind("127.0.0.1:9002").map_err(ServerError::Bind)?;

    let acceptor = TlsAcceptor::new(identity).map_err(ServerError::CreateTLSAcceptor)?;
    let acceptor = Arc::new(acceptor);

    for stream in server.incoming() {
        let stream = stream.map_err(ServerError::IncomingStream)?;
        let worker_id = next_worker_id();
        println!(
            "spawning worker with id {} for {:?}",
            worker_id,
            stream.local_addr()
        );
        let acceptor = acceptor.clone();
        Builder::new()
            .name(format!("worker-{}", worker_id))
            .spawn(move || {
                let stream = acceptor.accept(stream).unwrap();
                if let Err(err) = handle_client(worker_id, stream) {
                    eprintln!("error from worker thread {}: {:?}", worker_id, err)
                }
            })
            .map_err(ServerError::SpawnThread)?;
    }
    Ok(())
}

fn main() {
    run_server().unwrap();
}
