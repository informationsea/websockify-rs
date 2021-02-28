use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebsockifyError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Warp Error: {0}")]
    WarpError(#[from] warp::Error),
}
