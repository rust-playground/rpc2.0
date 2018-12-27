extern crate failure;
extern crate reqwest;
extern crate serde_json;

pub mod prelude {
    #[doc(no_inline)]
    pub use client::Client;
}

use error::{Error, ResponseError};

use std::fmt;
use std::mem;

use serde::de;
use serde::de::{DeserializeOwned, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use reqwest::header::{Accept, Authorization, Basic, ContentType, Headers, UserAgent};
use reqwest::mime;

const RPC_VERSION: &'static str = "2.0";

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

        // TODO: implement HTTP error derived from response code

        let results: Response<T> = res.json()?;
        if results.error.is_some() {
            return Err(Error::from(results.error.unwrap()));
        }
        let mut r = results.result;
        r.take().ok_or_else(|| {
            Error::from(ResponseError {
                code: 1,
                message: "blah".to_owned(),
            })
        })
        // Ok(mem::replace(&mut r, None).unwrap())
    }
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
