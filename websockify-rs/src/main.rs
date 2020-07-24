pub use warp_websockify::{WebsockifyError, WebsockifyErrorKind};

use clap::{crate_authors, crate_version, App, Arg};
use log::info;
use rust_embed::RustEmbed;
use std::env;
use std::net::ToSocketAddrs;
use tokio::net::UnixListener;
use warp::{http::Uri, Filter};

#[derive(RustEmbed)]
#[folder = "noVNC"]
struct NoVnc;

#[tokio::main]
async fn main() {
    let matches = App::new("WebSockify-rs")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Convert TCP/Unix doamin socket connection to WebSocket")
        .arg(
            Arg::with_name("upstream")
                .index(1)
                .help("Upstream host:port")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("listen")
                .index(2)
                .help("Listen host:port")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("upstream-unix")
                .short("u")
                .long("upstream-unix")
                .help("Upstream is unix domain socket"),
        )
        .arg(
            Arg::with_name("listen-unix")
                .short("l")
                .long("lisnten-unix")
                .help("Listen path is unix domain socket"),
        )
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .takes_value(true)
                .help("server prefix"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true),
        )
        .get_matches();

    match matches.occurrences_of("verbose") {
        1 => env::set_var("RUST_LOG", "info"),
        2 => env::set_var("RUST_LOG", "debug"),
        3 => env::set_var("RUST_LOG", "trace"),
        _ => {
            if env::var("RUST_LOG").is_err() {
                env::set_var("RUST_LOG", "warn")
            }
        }
    }
    pretty_env_logger::init();

    let upstream = if matches.is_present("upstream-unix") {
        warp_websockify::Destination::unix(matches.value_of("upstream").unwrap())
    } else {
        warp_websockify::Destination::tcp(matches.value_of("upstream").unwrap()).unwrap()
    };

    let ws = warp::path("websockify").and(warp_websockify::websockify(upstream));
    let static_file = warp::path("static").and(warp_embed::embed(&NoVnc {}));

    let server = static_file
        .with(warp::log("http"))
        .or(ws.with(warp::log("http")))
        .or(warp::path::end().map(|| warp::redirect(Uri::from_static("/vnc/static/vnc.html"))));

    let server = if let Some(x) = matches.value_of("prefix") {
        warp::path(x.to_string()).and(server).boxed()
    } else {
        server.boxed()
    };

    if matches.is_present("listen-unix") {
        let mut listener = UnixListener::bind(matches.value_of("listen").unwrap()).unwrap();
        let incoming = listener.incoming();
        warp::serve(server).run_incoming(incoming).await;
    } else {
        let listen = matches
            .value_of("listen")
            .unwrap()
            .to_socket_addrs()
            .unwrap();

        let binded: Vec<_> = listen
            .map(|x| {
                let binded = warp::serve(server.clone()).bind(x);
                info!("binded: {}", x);
                tokio::spawn(binded)
            })
            .collect();
        for one in binded {
            one.await.unwrap();
        }
    }
}
