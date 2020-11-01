use crate::{
    service::{auth, Config, Problem, ServiceProvider, Session, Urls},
    LeetUpError, Result,
};
use anyhow::anyhow;
use cookie::{Cookie, CookieJar};
use log::debug;
use reqwest::{
    blocking::{Body, Client, RequestBuilder, Response},
    header,
    header::HeaderMap,
    header::HeaderValue,
    header::CONTENT_TYPE,
    header::SET_COOKIE,
    StatusCode,
};
use serde_json::json;

fn headers_with_session(headers_opt: Option<HeaderMap>, session: Option<&Session>) -> HeaderMap {
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

    headers
}

/// Make a GET request
pub fn get(
    url: &str,
    headers_opt: Option<HeaderMap>,
    session: Option<&Session>,
) -> Result<Response> {
    let headers = headers_with_session(headers_opt, session);
    let client = Client::builder().default_headers(headers).build()?;
    client.get(url).send().map_err(LeetUpError::Reqwest)
}

/// Make a POST request
pub fn post<'a, P: ServiceProvider<'a>, T: serde::Serialize + ?Sized, F>(
    provider: &P,
    url: &str,
    body: &T,
    with_headers: F,
) -> Result<serde_json::value::Value>
where
    F: FnOnce() -> Option<HeaderMap>,
{
    let config = provider.config()?;
    let headers = headers_with_session(with_headers(), provider.session());
    let client = Client::builder().default_headers(headers).build()?;

    let client = client
        .post(url)
        .header(
            header::ORIGIN,
            HeaderValue::from_str(&config.urls.base).unwrap(),
        )
        .json(body);

    let res = client.send()?;

    if res.status() == 200 {
        res.json::<serde_json::value::Value>().map_err(|e| e.into())
    } else {
        log::error!("{:#?}", res);
        Err(LeetUpError::Any(anyhow!(format!(
            "Status: {}",
            res.status()
        ))))
    }
}
