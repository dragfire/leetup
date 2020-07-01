use crate::{LeetUpError, Result};
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
};

/// Make a GET request
pub fn get(url: &str, headers: HeaderMap) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    client.get(url).send().map_err(LeetUpError::Reqwest)
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
