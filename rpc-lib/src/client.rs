extern crate reqwest;
extern crate serde_json;

pub mod prelude {
    #[doc(no_inline)]
    pub use client::Client;
}

use std::error;
use std::fmt;
use std::fmt::Display;
use std::mem;

use serde::de;
use serde::de::{DeserializeOwned, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use reqwest::header::{Accept, Authorization, Basic, ContentType, Headers, UserAgent};
use reqwest::mime;

const RPC_VERSION: &'static str = "2.0";

/// Error represents the RPC error.
///
/// Error can be one of many types that can occur during a request.
///
#[derive(Debug)]
pub enum Error {
    Serialize(serde_json::Error),
    Request(reqwest::Error),
    RPC(ResponseError),
    Response(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serialize(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Request(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Serialize(ref err) => Display::fmt(err, f),
            Error::Request(ref err) => Display::fmt(err, f),
            Error::RPC(ref s) => f.write_str(format!("{:?}", s).as_ref()),
            Error::Response(ref err) => f.write_str(err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        ""
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::Serialize(ref err) => Some(err),
            Error::Request(ref err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Auth {
    username: String,
    password: Option<String>,
}

impl Auth {
    pub fn new(username: String, password: Option<String>) -> Self {
        Auth { username, password }
    }
}

/// Client contains all information and authentication to hit an RPC server
#[derive(Debug)]
pub struct Client<'a> {
    url: &'a str,
    auth: Option<Auth>,
    user_agent: String,
}

impl<'a> Client<'a> {
    pub fn new(url: &'a str) -> Self {
        Client {
            url,
            auth: None,
            user_agent: format!("rpc-lib/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn with_basic_auth(mut self, username: String, password: Option<String>) -> Self {
        self.auth = Some(Auth::new(username, password));
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn call<T>(&self, id: &'a str, method: &'a str, params: impl Serialize) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let req = Request::new(id, method, params);
        let j = serde_json::to_string(&req)?;
        let mut headers = Headers::new();
        headers.set(UserAgent::new(self.user_agent.clone()));
        headers.set(Accept::json());
        headers.set(ContentType(mime::APPLICATION_JSON));

        if self.auth.is_some() {
            let auth = self.auth.as_ref().unwrap();
            headers.set(Authorization(Basic {
                username: auth.username.clone(),
                password: auth.password.clone(),
            }));
        }

        let client = reqwest::Client::new();
        let mut res = client.post(self.url).headers(headers).body(j).send()?;

        let results: Response<T> = res.json()?;
        if results.error.is_some() {
            return Err(Error::RPC(results.error.unwrap()));
        }
        if results.result.is_none() {
            return Err(Error::Response(String::from(
                "invalid response, no result returned",
            )));
        }
        let mut r = results.result;
        Ok(mem::replace(&mut r, None).unwrap())
    }
}

/// ResponseError contains an RPC 2.0 error
#[derive(Debug, Deserialize)]
pub struct ResponseError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Response<T> {
    //    jsonrpc: String,
    result: Option<T>,
    #[serde(deserialize_with = "string_map_or_null")]
    error: Option<ResponseError>,
    id: String,
}

#[derive(Debug, Serialize)]
struct Request<'a, T> {
    jsonrpc: &'static str,
    method: &'a str,
    params: T,
    id: &'a str,
}

impl<'a, T> Request<'a, T>
where
    T: Serialize,
{
    pub fn new(id: &'a str, method: &'a str, params: T) -> Self {
        Request {
            jsonrpc: RPC_VERSION,
            method,
            params,
            id,
        }
    }
}

fn string_map_or_null<'de, D>(deserializer: D) -> Result<Option<ResponseError>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringMapOrNull;

    impl<'de> Visitor<'de> for StringMapOrNull {
        type Value = Option<ResponseError>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string, map, or null")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(ResponseError {
                code: -1,
                message: value.to_owned(),
            }))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let deserializer = de::value::MapAccessDeserializer::new(visitor);
            let response_error = ResponseError::deserialize(deserializer)?;
            Ok(Some(response_error))
        }
    }

    deserializer.deserialize_any(StringMapOrNull)
}
