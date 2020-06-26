use crate::{service, LeetUpError, Result};

const API_URI: &str = "https://leetcode.com/api";

/// Fetch URL
pub fn fetch_url(path: &str) -> Result<reqwest::blocking::Response> {
    let url = API_URI.to_string() + path;

    // make sure this also works with default headers
    let client = reqwest::blocking::Client::builder().build().unwrap();

    let res = client
        .get(&url)
        .header("Cookie", service::auth::Session::new().cookie)
        .send();

    res.map_err(LeetUpError::Reqwest)
}
