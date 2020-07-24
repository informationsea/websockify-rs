# warp-embed

Serve [embedded file](https://crates.io/crates/rust-embed) with [warp](https://crates.io/crates/warp)

```rust
use warp::Filter;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "data"]
struct Data;

let data_serve = warp_embed::embed(&Data);
```