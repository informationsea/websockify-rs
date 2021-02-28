# WebSockify-rs
[![Build](https://github.com/informationsea/websockify-rs/actions/workflows/build.yml/badge.svg)](https://github.com/informationsea/websockify-rs/actions/workflows/build.yml)

Rust implementation of [WebSockify](https://github.com/novnc/websockify).
noVNC files are embedded in websockify-rs for easy use.

## Usage

```
WebSockify-rs 0.1.0
Okamura Yasunobu
Convert TCP/Unix doamin socket connection to WebSocket

USAGE:
    websockify-rs [FLAGS] [OPTIONS] <upstream> <listen>

FLAGS:
    -h, --help             Prints help information
    -l, --listen-unix     Listen path is unix domain socket
    -u, --upstream-unix    Upstream is unix domain socket
    -V, --version          Prints version information
    -v, --verbose          

OPTIONS:
    -p, --prefix <prefix>    server prefix

ARGS:
    <upstream>    Upstream host:port
    <listen>      Listen host:port
```

## Example

```
$ vncserver
$ ./websockify-rs localhost:5901 127.0.0.1:8080
```

Access to [http://127.0.0.1/static/vnc.html](http://127.0.0.1/static/vnc.html)