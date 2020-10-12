//! # warp-embed
//!
//! Serve [embedded file](https://crates.io/crates/rust-embed) with [warp](https://crates.io/crates/warp)
//!
//! ```
//! use warp::Filter;
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed)]
//! #[folder = "data"]
//! struct Data;
//!
//! let data_serve = warp_embed::embed(&Data);
//! ```

use std::borrow::Cow;
use warp::{filters::path::Tail, reject::Rejection, reply::Reply, reply::Response, Filter};

struct EmbedFile {
    data: Cow<'static, [u8]>,
}

impl Reply for EmbedFile {
    fn into_response(self) -> Response {
        Response::new(self.data.into())
    }
}

fn append_filename(path: &str, filename: &str) -> String {
    if path.is_empty() {
        filename.to_string()
    } else {
        format!("{}/{}", path, filename)
    }
}

/// Creates a `Filter` that serves embedded files at the base `path` joined by the request path.
pub fn embed<A: rust_embed::RustEmbed>(
    _: &A,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::tail().and_then(|tail: Tail| async move {
        let (embedded_file, actual_name) = if let Some(x) = A::get(tail.as_str()) {
            (Some(x), tail.as_str())
        } else if let Some(x) = A::get(&append_filename(tail.as_str(), "index.html")) {
            (Some(x), "index.html")
        } else if let Some(x) = A::get(&append_filename(tail.as_str(), "index.htm")) {
            (Some(x), "index.html")
        } else {
            (None, "")
        };
        if let Some(x) = embedded_file {
            let suggest = mime_guess::guess_mime_type(actual_name);
            Ok(warp::reply::with_header(
                EmbedFile { data: x },
                "Content-Type",
                suggest.to_string(),
            ))
        } else {
            Err(warp::reject::not_found())
        }
    })
}

#[cfg(test)]
mod test;
