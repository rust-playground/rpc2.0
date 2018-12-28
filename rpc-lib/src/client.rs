use crate::error::{Error, ResponseError, Result as RpcResult};

use std::fmt;

use serde::{
    de,
    de::{DeserializeOwned, Deserializer, MapAccess, Visitor},
    Deserialize, Serialize,
};

use reqwest::{
    header,
    header::{HeaderMap, HeaderValue},
};

pub mod prelude {
    #[doc(no_inline)]
    pub use crate::client::Client;
}

const RPC_VERSION: &'static str = "2.0";

/// Client contains all information and authentication to hit an RPC server
#[derive(Debug)]
pub struct Client<'a> {
    client: reqwest::Client,
    url: &'a str,
    headers: HeaderMap,
}

impl<'a> Client<'a> {
    pub fn new(url: &'a str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_str(&format!("rpc-lib/{}", env!("CARGO_PKG_VERSION"))).unwrap(),
        );
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Client {
            client: reqwest::Client::new(),
            url,
            headers,
        }
    }

    pub fn with_basic_auth(mut self, username: &'a str, password: Option<&'a str>) -> Self {
        let auth = match password {
            Some(password) => format!("{}:{}", username, password),
            None => format!("{}:", username),
        };
        let header_value = format!("Basic {}", base64::encode(&auth));
        self.headers
            .insert(header::AUTHORIZATION, header_value.parse().unwrap());
        self
    }

    pub fn with_user_agent<S>(mut self, user_agent: S) -> Self
    where
        S: Into<String>,
    {
        self.headers
            .insert(header::USER_AGENT, user_agent.into().parse().unwrap());
        self
    }

    pub fn call<T>(&self, id: &'a str, method: &'a str, params: impl Serialize) -> RpcResult<T>
    where
        T: DeserializeOwned,
    {
        let req = Request::new(id, method, params);
        let mut results: Response<T> = self
            .client
            .post(self.url)
            .headers(self.headers.clone())
            .json(&req)
            .send()?
            .json()?;

        if results.error.is_some() {
            return Err(Error::from(results.error.unwrap()));
        }
        results.result.take().ok_or_else(|| {
            Error::from(ResponseError {
                code: -2,
                message: "invalid response".to_owned(),
            })
        })
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
