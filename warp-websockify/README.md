# warp-websockify
websockify implementation for warp

```rust
use warp::Filter;
use warp_websockify::{Destination, websockify};

let dest = Destination::tcp("localhost:5901").unwrap();
let serve = websockify(dest);
```