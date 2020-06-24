use crate::{LeetUpError, Result};

const API_URI: &str = "https://leetcode.com/api";

/// Fetch URL
pub fn fetch_url(path: &str) -> Result<reqwest::blocking::Response> {
    let url = API_URI.to_string() + path;
    reqwest::blocking::get(&url).map_err(LeetUpError::Reqwest)
}
