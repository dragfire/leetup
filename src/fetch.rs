use crate::Result;
use request::{Client, List, Response};

/// Make a GET request
pub fn get(url: &str, headers: List) -> Result<Response> {
    let client = Client::builder()
        .default_headers(headers)
        .redirect(true)
        .build();

    Ok(client.get(url).perform())
}
