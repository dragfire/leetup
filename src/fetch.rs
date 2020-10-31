use crate::{
    service::{auth, Config, ServiceProvider, Session, Urls},
    LeetUpError, Result,
};
use anyhow::anyhow;
use log::debug;
use request::{Client, List, Response};
use serde_json::json;

#[derive(Debug)]
pub struct Problem {
    pub id: usize,
    pub slug: String,
    pub lang: String,
    pub link: String,
}

/// Make a GET request
pub fn get(url: &str, headers: Option<List>, session: Option<&Session>) -> Result<Response> {
    let mut client = Client::builder().redirect(true);
    if let Some(headers) = headers {
        client = client.default_headers(headers);
    }
    let client = client.build();
    let mut client = client.get(url);
    if let Some(session) = session {
        let cookie: String = session.into();
        client = client.cookie(cookie);
    }
    Ok(client.perform())
}

/// Make a POST request
pub fn post<'a, P: ServiceProvider<'a>>(
    provider: &P,
    url: &str,
    problem: Problem,
    body: String,
) -> Result<serde_json::value::Value> {
    let config = provider.config()?;
    let client = request::Client::builder().redirect(true).build();
    let session = provider.session().ok_or_else(|| LeetUpError::OptNone)?;
    let cookie_header: String = session.into();
    let csrf = &session.csrf;

    let client = client
        .post(url)
        .referer(problem.link)
        .cookie(cookie_header)
        .header("Host: leetcode.com")
        .header(&format!("x-csrftoken: {}", csrf))
        .header("X-Requested-With: XMLHttpRequest")
        .header("Content-Type: application/json")
        .header("Origin: https://leetcode.com")
        .body(body);

    let res = client.perform();

    if res.status() == 200 {
        res.json::<serde_json::value::Value>().map_err(|e| e.into())
    } else {
        Err(LeetUpError::Any(anyhow!(format!(
            "Status: {}",
            res.status()
        ))))
    }
}
