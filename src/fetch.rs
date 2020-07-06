use crate::{LeetUpError, Result};
use request::{Client, ClientBuilder, List, Request, RequestBuilder, Response};
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
};

/// Make a GET request
pub fn get(url: &str, headers: List) -> Result<Response> {
    let client = Client::builder().default_headers(headers).build();

    Ok(client.get(url).perform())
}

/// Make a POST request
pub fn post(
    url: &str,
    headers: HeaderMap<HeaderValue>,
    form: Vec<(&str, &str)>,
) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let req = client
        .post(url)
        .form(&form)
        .header("jar", "true")
        .header("User-Agent", "Leetup");
    println!("{:?}", req);

    req.send().map_err(LeetUpError::Reqwest)
}
