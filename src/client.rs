use crate::{
    service::{auth, Config, Problem, ServiceProvider, Session, Urls},
    LeetUpError, Result,
};
use anyhow::anyhow;
use cookie::{Cookie, CookieJar};
use log::debug;
use reqwest::{
    blocking::{Body, Client, RequestBuilder, Response},
    header::HeaderMap,
    header::HeaderValue,
    header::CONTENT_TYPE,
    header::SET_COOKIE,
    StatusCode,
};
use serde_json::json;

/// Make a GET request
pub fn get(
    url: &str,
    headers_opt: Option<HeaderMap>,
    session: Option<&Session>,
) -> Result<Response> {
    let mut headers = HeaderMap::new();
    if let Some(h) = headers_opt {
        headers = HeaderMap::from(h);
    }

    if let Some(session) = session {
        let cookie: String = session.into();
        headers.insert("Cookie", HeaderValue::from_str(&cookie).unwrap());
        headers.insert("X-CSRFToken", HeaderValue::from_str(&session.csrf).unwrap());
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest"),
        );
    }

    let client = Client::builder().default_headers(headers).build()?;
    client.get(url).send().map_err(LeetUpError::Reqwest)
}

/// Make a POST request
pub fn post<'a, P: ServiceProvider<'a>>(
    provider: &P,
    url: &str,
    problem: Problem,
    body: String,
) -> Result<serde_json::value::Value> {
    //let config = provider.config()?;
    //let client = reqwest::Client::builder().build()?;
    //let session = provider.session().ok_or_else(|| LeetUpError::OptNone)?;
    //let cookie_header: String = session.into();
    //let csrf = &session.csrf;

    //let client = client
    //    .post(url)
    //    .header("Host: leetcode.com")
    //    .header(&format!("x-csrftoken: {}", csrf))
    //    .header("X-Requested-With: XMLHttpRequest")
    //    .header("Content-Type: application/json")
    //    .header("Origin: https://leetcode.com")
    //    .body(body);

    //let res = client.perform();

    //if res.status() == 200 {
    //    res.json::<serde_json::value::Value>().map_err(|e| e.into())
    //} else {
    //    Err(LeetUpError::Any(anyhow!(format!(
    //        "Status: {}",
    //        res.status()
    //    ))))
    //}
    Ok(json!({}))
}
