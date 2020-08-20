//! # warp-websockify
//! websockify implementation for warp
//!
//! ```
//! use warp::Filter;
//! use warp_websockify::{Destination, websockify};
//!
//! let dest = Destination::tcp("localhost:5901").unwrap();
//! let serve = websockify(dest);
//! ```

mod error;

pub use error::{WebsockifyError, WebsockifyErrorKind};
use futures::prelude::*;
use log::{debug, error, info};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};
use warp::ws::{Message, WebSocket, Ws};
use warp::{reject::Rejection, reply::Reply, Filter};

fn option_socket_to_string(addr: Option<SocketAddr>) -> String {
    if let Some(addr) = addr {
        format!("{}", addr)
    } else {
        "[NO ADDR]".to_string()
    }
}

/// WebSockify upstream
pub enum Destination {
    /// Connect to TCP
    Tcp(Vec<SocketAddr>),

    /// Connect to unix domain socket
    Unix(PathBuf),
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Destination::Tcp(tcp) => write!(f, "{:?}", tcp),
            Destination::Unix(unix) => write!(f, "{}", unix.to_str().unwrap()),
        }
    }
}

impl Destination {
    /// Create destination to unix domain socket
    pub fn unix<P: AsRef<Path>>(path: P) -> Destination {
        Destination::Unix(path.as_ref().to_path_buf())
    }

    /// Create destination to TCP
    pub fn tcp(addr: impl ToSocketAddrs) -> io::Result<Destination> {
        Ok(Destination::Tcp(addr.to_socket_addrs()?.collect()))
    }

    async fn connect(&self) -> io::Result<NetStream> {
        match self {
            Destination::Tcp(addrs) => {
                let mut last_error = None;
                for one in addrs {
                    match TcpStream::connect(one).await {
                        Ok(stream) => return Ok(NetStream::Tcp(stream)),
                        Err(e) => last_error = Some(e),
                    }
                }
                Err(last_error.unwrap())
            }
            Destination::Unix(path) => Ok(NetStream::Unix(UnixStream::connect(path).await?)),
        }
    }
}

enum NetStream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

/// Creates a `Filter` that connet to TCP or unix domain socket.
pub fn websockify(
    dest: Destination,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let dest: Arc<Destination> = Arc::new(dest);
    let dest = warp::any().map(move || dest.clone());
    warp::addr::remote()
        .and(warp::ws())
        .and(dest)
        .and_then(websockify_connect)
}

/// Connect to TCP or unix domain socket and redirect to websocket
pub fn websockify_connect(
    addr: Option<SocketAddr>,
    ws: Ws,
    dest: impl AsRef<Destination>,
) -> impl TryFuture<Ok = impl Reply, Error = Rejection> {
    async move {
        match dest.as_ref().connect().await {
            Ok(stream) => {
                info!(
                    "{} target:[{}] Connection started",
                    option_socket_to_string(addr),
                    dest.as_ref()
                );
                Ok(ws.on_upgrade(move |ws| connect(addr, ws, stream)))
            }
            Err(e) => {
                error!(
                    "{} target:[{}] {}",
                    option_socket_to_string(addr),
                    dest.as_ref(),
                    e
                );
                Err(warp::reject::reject())
            }
        }
    }
}

async fn connect(addr: Option<SocketAddr>, ws: WebSocket, stream: NetStream) {
    match stream {
        NetStream::Unix(x) => unix_connect(addr, ws, x).await,
        NetStream::Tcp(x) => tcp_connect(addr, ws, x).await,
    }
}

async fn tcp_connect(addr: Option<SocketAddr>, ws: WebSocket, mut stream: TcpStream) {
    let (tcp_rx, tcp_tx) = stream.split();
    let (tx, rx) = ws.split();

    let x = handle_ws_rx(addr, rx, tcp_tx);
    let y = handle_ws_tx(addr, tx, tcp_rx);

    if let Err(e) = tokio::try_join!(x, y) {
        error!(
            "{} tcp connection error: {}",
            option_socket_to_string(addr),
            e
        );
    }
}

async fn unix_connect(addr: Option<SocketAddr>, ws: WebSocket, mut stream: UnixStream) {
    let (tcp_rx, tcp_tx) = stream.split();
    let (tx, rx) = ws.split();

    let x = handle_ws_rx(addr, rx, tcp_tx);
    let y = handle_ws_tx(addr, tx, tcp_rx);
    if let Err(e) = tokio::try_join!(x, y) {
        error!(
            "{} unix connection error: {}",
            option_socket_to_string(addr),
            e
        );
    }
}

async fn handle_ws_rx<
    WSR: Stream<Item = Result<Message, warp::Error>> + std::marker::Unpin + std::marker::Send,
    SW: AsyncWrite + std::marker::Unpin + std::marker::Send,
>(
    addr: Option<SocketAddr>,
    mut rx: WSR,
    mut tcp_tx: SW,
) -> Result<(), WebsockifyError> {
    while let Some(message) = rx.next().await {
        let message = message?;
        if message.is_binary() {
            tcp_tx.write_all(message.as_bytes()).await?;
        }
    }
    info!("{} WebSocket was closed", option_socket_to_string(addr));
    Ok(())
}

async fn handle_ws_tx<
    WST: Sink<Message, Error = warp::Error> + std::marker::Unpin + std::marker::Send,
    SR: AsyncRead + std::marker::Unpin + std::marker::Send,
>(
    addr: Option<SocketAddr>,
    mut tx: WST,
    mut tcp_rx: SR,
) -> Result<(), WebsockifyError> {
    let mut buffer: Vec<u8> = Vec::with_capacity(10000);
    for _ in 0..10000 {
        buffer.push(0);
    }

    loop {
        let read_bytes = tcp_rx.read(&mut buffer).await?;
        if read_bytes > 0 {
            //println!("send bytes: {}", read_bytes);
            tx.send(Message::binary(&buffer[0..read_bytes])).await?;
        } else {
            debug!("{} read zero bytes", option_socket_to_string(addr));
            tx.close().await?;
            break;
        }
    }

    info!(
        "{} destination socket was closed",
        option_socket_to_string(addr)
    );
    Ok(())
}
