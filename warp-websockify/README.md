# warp-websockify
[![Build Status](https://travis-ci.org/informationsea/websockify-rs.svg?branch=master)](https://travis-ci.org/informationsea/websockify-rs)
[![GitHub](https://img.shields.io/github/license/informationsea/websockify-rs)](https://github.com/informationsea/websockify-rs)
[![GitHub top language](https://img.shields.io/github/languages/top/informationsea/websockify-rs)](https://github.com/informationsea/websockify-rs)
[![Crates.io](https://img.shields.io/crates/v/warp-websockify)](https://crates.io/crates/warp-websockify)
[![Docs.rs](https://docs.rs/warp-websockify/badge.svg)](https://docs.rs/warp-websockify)

websockify implementation for warp

```rust
use warp::Filter;
use warp_websockify::{Destination, websockify};

let dest = Destination::tcp("localhost:5901").unwrap();
let serve = websockify(dest);
```