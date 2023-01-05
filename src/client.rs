use crate::{service::Session, Config, LeetUpError, Result};
use anyhow::anyhow;
use reqwest::{header, header::HeaderMap, header::HeaderValue, Client, Response};

pub struct RemoteClient<'a> {
    config: &'a Config,
    session: Option<&'a Session>,
}

impl<'a> RemoteClient<'_> {
    pub fn new(config: &'a Config, session: Option<&'a Session>) -> RemoteClient<'a> {
        RemoteClient { config, session }
    }

    /// Make a GET request
    pub async fn get(
        &self,
        url: &str,
        headers_opt: Option<HeaderMap>,
        session: Option<&Session>,
    ) -> Result<Response> {
        let headers = self.headers_with_session(headers_opt, session);
        let client = Client::builder().default_headers(headers).build()?;
        client.get(url).send().await.map_err(LeetUpError::Reqwest)
    }

    /// Make a POST request
    pub async fn post<T: serde::Serialize + ?Sized, F>(
        &self,
        url: &str,
        body: &T,
        with_headers: F,
    ) -> Result<serde_json::value::Value>
    where
        F: FnOnce() -> Option<HeaderMap>,
    {
        let headers = self.headers_with_session(with_headers(), self.session);
        let client = Client::builder().default_headers(headers).build()?;

        let client = client
            .post(url)
            .header(
                header::ORIGIN,
                HeaderValue::from_str(&self.config.urls.base).unwrap(),
            )
            .json(body);

        let res = client.send().await?;

        if res.status() == 200 {
            res.json::<serde_json::value::Value>()
                .await
                .map_err(|e| e.into())
        } else {
            Err(LeetUpError::Any(anyhow!(
                "Status: {}",
                res.status()
            )))
        }
    }

    fn headers_with_session(
        &self,
        headers_opt: Option<HeaderMap>,
        session: Option<&Session>,
    ) -> HeaderMap {
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
}
