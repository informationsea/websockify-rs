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

pub use error::WebsockifyError;
use futures::prelude::*;
use log::{debug, error, info};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use warp::ws::{Message, WebSocket, Ws};
use warp::{reject::Rejection, reply::Reply, Filter};
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use tokio::net::UnixStream;

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
    #[cfg(unix)]
    Unix(PathBuf),
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Destination::Tcp(tcp) => write!(f, "{:?}", tcp),
            #[cfg(unix)]
            Destination::Unix(unix) => write!(f, "{}", unix.to_str().unwrap()),
        }
    }
}

impl Destination {
    /// Create destination to unix domain socket
    #[cfg(unix)]
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
            #[cfg(unix)]
            Destination::Unix(path) => Ok(NetStream::Unix(UnixStream::connect(path).await?)),
        }
    }
}

enum NetStream {
    #[cfg(unix)]
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
    if let Err(e) = match stream {
        #[cfg(unix)]
        NetStream::Unix(x) => unified_connect(addr, ws, x).await,
        NetStream::Tcp(x) => unified_connect(addr, ws, x).await,
    } {
        error!("{}: Error: {}", option_socket_to_string(addr), e);
    }
    //warn!("Connection finished");
}

async fn unified_connect<S>(
    addr: Option<SocketAddr>,
    mut ws: WebSocket,
    mut stream: S,
) -> Result<(), WebsockifyError>
where
    S: AsyncRead + AsyncWrite + std::marker::Unpin,
{
    let mut buffer: Vec<u8> = vec![0; 10000];
    loop {
        tokio::select! {
            message = ws.next() => {
                if let Some(message) = message {
                    let message = message?;
                    if message.is_binary() {
                        stream.write_all(message.as_bytes()).await?;
                    } else if message.is_close () {
                        if let Some((code, reason)) = message.close_frame() {
                            debug!("{}: Web socket closed: {}: {}", option_socket_to_string(addr), code, reason);
                        } else {
                            debug!("{}: Web socket closed", option_socket_to_string(addr))
                        }
                        return Ok(());
                    }
                } else {
                    error!("{}: No packet received from websocket", option_socket_to_string(addr));
                }
            },
            data_bytes = stream.read(&mut buffer) => {
                let data_bytes = data_bytes?;
                if data_bytes > 0 {
                    ws.send(Message::binary(&buffer[0..data_bytes])).await?;
                } else {
                    debug!("{}: TCP/Unix stream closed", option_socket_to_string(addr));
                    ws.close().await?;
                    return Ok(());
                }
            }
        }
    }
}
