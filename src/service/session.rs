use std::str::FromStr;

use cookie::Cookie;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub id: String,
    pub csrf: String,
}

impl Session {
    pub fn new(id: String, csrf: String) -> Self {
        Session { id, csrf }
    }
}

impl FromStr for Session {
    type Err = cookie::ParseError;

    fn from_str(raw: &str) -> std::result::Result<Self, Self::Err> {
        let raw_split = raw.split("; ");

        let cookies = raw_split.map(Cookie::parse).collect::<Vec<_>>();
        let mut id = String::new();
        let mut csrf = String::new();

        for cookie in cookies {
            let cookie = cookie?;
            let name = cookie.name();
            match name {
                "LEETCODE_SESSION" => id = cookie.value().to_string(),
                "csrftoken" => csrf = cookie.value().to_string(),
                _ => (),
            }
        }

        Ok(Session { id, csrf })
    }
}

fn session_to_cookie(id: &str, csrf: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}={}; ", "LEETCODE_SESSION", id));
    s.push_str(&format!("{}={}", "csrftoken", csrf));

    s
}

impl From<Session> for String {
    fn from(session: Session) -> Self {
        session_to_cookie(&session.id, &session.csrf)
    }
}

impl From<&Session> for String {
    fn from(session: &Session) -> Self {
        session_to_cookie(&session.id, &session.csrf)
    }
}

#[test]
fn test_cookie_parser() {
    let cookie = "csrftoken=asdsd; LEETCODE_SESSION=asdasd";
    let session: Session = Session::from_str(cookie).unwrap();

    assert!(!session.csrf.is_empty());
    assert!(!session.id.is_empty());
}
