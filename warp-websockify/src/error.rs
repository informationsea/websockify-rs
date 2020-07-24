use failure::{Context, Fail};

#[derive(Debug)]
pub struct WebsockifyError {
    inner: Context<WebsockifyErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum WebsockifyErrorKind {
    #[fail(display = "IO Error")]
    IoError,
    #[fail(display = "Warp Error")]
    WarpError,
}

impl Fail for WebsockifyError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl std::fmt::Display for WebsockifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl WebsockifyError {
    pub fn kind(&self) -> WebsockifyErrorKind {
        *self.inner.get_context()
    }
}

impl From<WebsockifyErrorKind> for WebsockifyError {
    fn from(kind: WebsockifyErrorKind) -> WebsockifyError {
        WebsockifyError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<WebsockifyErrorKind>> for WebsockifyError {
    fn from(inner: Context<WebsockifyErrorKind>) -> WebsockifyError {
        WebsockifyError { inner }
    }
}

impl From<std::io::Error> for WebsockifyError {
    fn from(error: std::io::Error) -> WebsockifyError {
        error.context(WebsockifyErrorKind::IoError).into()
    }
}

impl From<warp::Error> for WebsockifyError {
    fn from(error: warp::Error) -> WebsockifyError {
        error.context(WebsockifyErrorKind::WarpError).into()
    }
}
