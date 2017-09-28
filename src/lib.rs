//! Goji provides an interface for Jira's REST api

#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use std::io::Read;

use reqwest::header::{Authorization, Basic, Cookie};
use reqwest::{Client, Method, StatusCode};
use reqwest::header::ContentType;
use serde::Serialize;
use serde::de::DeserializeOwned;

mod transitions;
pub use transitions::*;
mod issues;
pub use issues::*;
mod search;
pub use search::Search;
mod builder;
pub use builder::*;
mod errors;
pub use errors::*;
mod rep;
pub use rep::*;

pub type Result<T> = std::result::Result<T, Error>;

/// Types of authentication credentials
#[derive(Clone, Debug)]
pub enum Credentials {
    /// username and password credentials
    Basic(String, String), // todo: OAuth
    /// Cookie auth via JSESSIONID
    Cookie(String),
}

/// Entrypoint into client interface
/// https://docs.atlassian.com/jira/REST/latest/
#[derive(Clone, Debug)]
pub struct Jira {
    host: String,
    credentials: Credentials,
    client: Client,
}

impl Jira {
    /// creates a new instance of a jira client
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Jira>
    where
        H: Into<String>,
    {
        Ok(Jira {
            host: host.into(),
            credentials: credentials,
            client: Client::new()?,
        })
    }

    /// creates a new instance of a jira client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: Credentials, client: Client) -> Result<Jira>
    where
        H: Into<String>,
    {
        Ok(Jira {
            host: host.into(),
            credentials,
            client,
        })
    }

    /// return transitions interface
    pub fn transitions<K>(&self, key: K) -> Transitions
    where
        K: Into<String>,
    {
        Transitions::new(self, key)
    }

    /// return search interface
    pub fn search(&self) -> Search {
        Search::new(self)
    }

    // return issues interface
    pub fn issues(&self) -> Issues {
        Issues::new(self)
    }

    fn post<D, S>(&self, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = try!(serde_json::to_string::<S>(&body));
        self.request::<D>(Method::Post, endpoint, Some(data.into_bytes()))
    }

    fn get<D>(&self, endpoint: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request::<D>(Method::Get, endpoint, None)
    }

    fn request<D>(&self, method: Method, endpoint: &str, body: Option<Vec<u8>>) -> Result<D>
    where
        D: DeserializeOwned,
    {
        let url = format!("{}/rest/api/latest{}", self.host, endpoint);
        debug!("url -> {:?}", url);

        let mut req = self.client.request(method, &url)?;
        let builder = match self.credentials {
            Credentials::Basic(ref user, ref pass) => {
                req.header(Authorization(Basic {
                    username: user.to_owned(),
                    password: Some(pass.to_owned()),
                })).header(ContentType::json())
            }
            Credentials::Cookie(ref jsessionid) => {
                let mut cookie = Cookie::new();
                cookie.append("JSESSIONID", jsessionid.to_owned());
                req.header(cookie)
                    .header(ContentType::json())
            }
        };

        let mut res = try!(match body {
            Some(bod) => builder.body(bod).send(),
            _ => builder.send(),
        });

        let mut body = String::new();
        try!(res.read_to_string(&mut body));
        debug!("status {:?} body '{:?}'", res.status(), body);
        match res.status() {
            StatusCode::Unauthorized => {
                // returns unparsable html
                Err(Error::Unauthorized)
            }
            client_err if client_err.is_client_error() => {
                Err(Error::Fault {
                    code: res.status(),
                    errors: try!(serde_json::from_str::<Errors>(&body)),
                })
            }
            _ => Ok(try!(serde_json::from_str::<D>(&body))),
        }
    }
}
