use super::{Error, Jira, Result};
use reqwest::Method;
extern crate serde_json;


#[derive(Deserialize, Debug)]
pub struct Login {
    pub session: SessionCookie,
    #[serde(rename = "loginInfo")]
    pub login_info: LoginInfo,
}

#[derive(Deserialize, Debug)]
pub struct CurrentUser {
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(rename = "loginInfo")]
    pub login_info: LoginInfo,
}

#[derive(Deserialize, Debug)]
pub struct SessionCookie {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginInfo {
    #[serde(rename = "failedLoginCount")]
    pub failed_login_count: u64,
    #[serde(rename = "loginCount")]
    pub login_count: u64,
    #[serde(rename = "lastFailedLoginTime")]
    pub last_failed_login_time: String,
    #[serde(rename = "previousLoginTime")]
    pub previous_login_time: String,
}

#[derive(Serialize, Debug)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}


/// Session options
#[derive(Debug)]
pub struct Session {
    jira: Jira,
}

impl Session {
    pub fn new(jira: &Jira) -> Session {
        Session { jira: jira.clone() }
    }

    pub fn login(&self, creds: LoginCredentials) -> Result<Login> {
        let data = try!(serde_json::to_string(&creds));
        self.jira.request(Method::Post, "/session", "auth", Some(data.into_bytes()))
    }

    pub fn logout(&self) -> Result<()> {
        self.jira.request(Method::Delete, "/session", "auth", None)
            .or_else(|e| match e {
                Error::Serde(_) => Ok(()),
                e => Err(e),
            })
    }

    pub fn current_user(&self) -> Result<CurrentUser> {
        self.jira.request(Method::Get, "/session", "auth", None)
    }
}
