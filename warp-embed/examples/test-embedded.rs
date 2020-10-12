use clap::{crate_authors, crate_version, App, Arg};
use log::info;
use rust_embed::RustEmbed;
use std::env;
use std::net::ToSocketAddrs;
use warp::Filter;

#[derive(RustEmbed)]
#[folder = "data"]
struct Data;

#[tokio::main]
async fn main() {
    let matches = App::new("Embedded test")
        .version(crate_version!())
        .author(crate_authors!())
        .about("warp-embed test server")
        .arg(
            Arg::with_name("listen")
                .index(1)
                .help("Listen host:port")
                .takes_value(true)
                .required(true),
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

    let static_file = warp_embed::embed(&Data {});

    let server = static_file.with(warp::log("http"));

    let server = if let Some(x) = matches.value_of("prefix") {
        warp::path(x.to_string()).and(server).boxed()
    } else {
        server.boxed()
    };

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
