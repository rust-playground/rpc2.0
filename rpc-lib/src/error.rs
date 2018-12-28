use failure::{Backtrace, Context, Fail};
use std::{fmt,fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn new(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum ErrorKind {
    #[fail(display = "HTTP error: {}", _0)]
    Http(String),
    #[fail(display = "Serialization error: {}", _0)]
    Serialization(String),
    #[fail(display = "Server error: {}", _0)]
    Server(String),
    #[fail(display = "Client error: {}", _0)]
    Client(String),
    #[fail(display = "Response error: {}", _0)]
    Response(String),
    #[fail(display = "Uncategorized error: {}", _0)]
    Unhandled(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        if err.is_http() {
            Error {
                inner: Context::new(ErrorKind::Http(format!("{}", err))),
            }
        } else if err.is_serialization() {
            Error {
                inner: Context::new(ErrorKind::Serialization(format!("{}", err))),
            }
        } else if err.is_redirect() || err.is_server_error() {
            Error {
                inner: Context::new(ErrorKind::Server(format!("{}", err))),
            }
        } else if err.is_client_error() {
            Error {
                inner: Context::new(ErrorKind::Client(format!("{}", err))),
            }
        } else {
            Error {
                inner: Context::new(ErrorKind::Unhandled(format!("{}", err))),
            }
        }
    }
}

/// ResponseError contains an RPC 2.0 error
#[derive(Debug, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl From<ResponseError> for Error {
    fn from(err: ResponseError) -> Error {
        Error {
            inner: Context::new(ErrorKind::Response(format!("{}", err))),
        }
    }
}
