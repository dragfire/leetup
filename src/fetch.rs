use crate::{service::ServiceProvider, LeetUpError, Result};
use reqwest::header;

/// Make a GET request
pub fn get<'a, P: ServiceProvider<'a>>(
    url: &str,
    provider: &P,
) -> Result<reqwest::blocking::Response> {
    let session = provider.session();
    let mut headers = header::HeaderMap::new();
    if let Some(sess) = session {
        let cookie = sess.cookie.parse().unwrap();
        headers.insert("Cookie", cookie);
    }

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    client.get(url).send().map_err(LeetUpError::Reqwest)
}
