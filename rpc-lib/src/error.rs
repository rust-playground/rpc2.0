extern crate failure;

// TODO: add macro for imp From<...> for Error

use failure::{Backtrace, Context, Fail};
use reqwest::Error as RequestError;
use serde_json::Error as JSONError;
use std::error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "JSON parsing error")]
    JSON,
    #[fail(display = "RPC request failed")]
    RPC,
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

impl From<RequestError> for Error {
    fn from(err: RequestError) -> Error {
        Error {
            inner: err.context(ErrorKind::RPC),
        }
    }
}

impl From<JSONError> for Error {
    fn from(err: JSONError) -> Error {
        Error {
            inner: err.context(ErrorKind::RPC),
        }
    }
}

/// ResponseError contains an RPC 2.0 error
#[derive(Debug, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
}

impl error::Error for ResponseError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl From<ResponseError> for Error {
    fn from(err: ResponseError) -> Error {
        Error {
            inner: err.context(ErrorKind::RPC),
        }
    }
}
