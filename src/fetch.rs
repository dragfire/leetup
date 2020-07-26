use crate::{
    service::{auth, Config, ServiceProvider, Session, Urls},
    LeetUpError, Result,
};
use request::{Client, List, Response};
use serde_json::json;

#[derive(Debug)]
pub struct Problem {
    pub link: String,
    pub slug: String,
}

/// Make a GET request
pub fn get(url: &str, headers: List) -> Result<Response> {
    let client = Client::builder()
        .default_headers(headers)
        .redirect(true)
        .build();

    Ok(client.get(url).perform())
}

/// Make graphql request
pub fn graphql_request<'a, P: ServiceProvider<'a>>(
    provider: &P,
    problem: Problem,
    body: String,
) -> Result<()> {
    let config = provider.config()?;
    let client = request::Client::builder()
        .http2(true)
        .redirect(true)
        .build();
    let body = body.to_string();
    let session = provider.session().ok_or_else(|| LeetUpError::OptNone)?;
    let cookie_header: String = session.into();
    let csrf = &session.csrf;

    let res = client
        .post(&config.urls.graphql)
        .referer(problem.link)
        .cookie(cookie_header)
        .header("Host: leetcode.com")
        .header(&format!("x-csrftoken: {}", csrf))
        .header("X-Requested-With: XMLHttpRequest")
        .header("Content-Type: application/json")
        .header("Origin: https://leetcode.com")
        .body(body)
        .perform();
    let data = res.json::<serde_json::value::Value>().unwrap();

    println!("{:?}", data);

    Ok(())
}
