use bytes::Bytes;
use curl::easy::Easy;
pub use curl::easy::List;
use serde::de::DeserializeOwned;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    url: PathBuf,
    headers: List,
    body: Option<Bytes>,
}

impl Request {
    pub fn new<T: AsRef<Path>>(method: Method, url: T) -> Self {
        Request {
            method,
            url: url.as_ref().to_owned(),
            headers: List::new(),
            body: None,
        }
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    pub fn url(&self) -> &PathBuf {
        &self.url
    }

    pub fn url_mut(&mut self) -> &mut PathBuf {
        &mut self.url
    }

    pub fn headers(&self) -> &List {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut List {
        &mut self.headers
    }
    pub fn body(&self) -> Option<&Bytes> {
        self.body.as_ref()
    }

    pub fn body_mut(&mut self) -> Option<&mut Bytes> {
        self.body.as_mut()
    }
}

pub struct RequestBuilder {
    client: Client,
    request: Request,
}

impl RequestBuilder {
    pub fn new(client: Client, request: Request) -> Self {
        RequestBuilder { client, request }
    }

    pub fn header(mut self, header: &str) -> Self {
        self.request
            .headers_mut()
            .append(header)
            .expect("Unable to add header");
        self
    }

    pub fn body(mut self, body: Bytes) -> Self {
        *self.request.body_mut().unwrap() = body;
        self
    }

    pub fn build(self) -> Request {
        self.request
    }

    pub fn perform(mut self) -> Response {
        self.client.perform(self.request)
    }
}

/// Client wraps libcurl Easy
pub struct ClientBuilder {
    headers: List,
    cookie_jar: bool,
    redirect: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        let mut headers = List::new();
        headers.append("Accept: */*").unwrap();

        ClientBuilder {
            cookie_jar: false,
            headers,
            redirect: false,
        }
    }

    pub fn default_headers(mut self, headers: List) -> Self {
        self.headers = headers;
        self
    }

    pub fn redirect(mut self, enabled: bool) -> Self {
        self.redirect = enabled;
        self
    }

    pub fn build(self) -> Client {
        let mut handle = Easy::new();
        if self.cookie_jar {
            handle.cookie_jar("cookie").unwrap();
        }

        handle.http_headers(self.headers).unwrap();
        handle.follow_location(self.redirect).unwrap();

        Client { handle }
    }
}

pub struct Client {
    handle: Easy,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn get<R: AsRef<Path>>(self, url: R) -> RequestBuilder {
        self.request(Method::Get, url)
    }

    pub fn post<R: AsRef<Path>>(self, url: R) -> RequestBuilder {
        self.request(Method::Post, url)
    }

    pub fn put<R: AsRef<Path>>(self, url: R) -> RequestBuilder {
        self.request(Method::Put, url)
    }

    pub fn request<R: AsRef<Path>>(self, method: Method, url: R) -> RequestBuilder {
        let req = Request::new(method, url);
        RequestBuilder::new(self, req)
    }

    pub fn perform(&mut self, request: Request) -> Response {
        let mut headers = Vec::new();
        let mut buf = Vec::new();
        self.handle.url(request.url.to_str().unwrap()).unwrap();

        match request.method() {
            Method::Get => self.handle.get(true).unwrap(),
            Method::Post => self.handle.post(true).unwrap(),
            _ => (),
        }

        {
            let mut transfer = self.handle.transfer();
            transfer
                .read_function(|buf| Ok(request.body().unwrap().as_ref().read(buf).unwrap_or(0)))
                .unwrap();
            transfer
                .write_function(|data| {
                    buf.extend_from_slice(data);
                    Ok(data.len())
                })
                .unwrap();
            transfer
                .header_function(|header| {
                    headers.push(std::str::from_utf8(header).unwrap().to_string());
                    true
                })
                .unwrap();
            transfer.perform().unwrap();
        }

        let body = if buf.len() == 0 {
            None
        } else {
            Some(Bytes::copy_from_slice(&buf))
        };

        let status = self.handle.response_code().unwrap();

        Response::new(headers, body, status)
    }
}

#[derive(Debug)]
pub struct Response {
    headers: Vec<String>,
    body: Option<Bytes>,
    status: u32,
}

impl Response {
    pub fn new(headers: Vec<String>, body: Option<Bytes>, status: u32) -> Self {
        Response {
            headers,
            body,
            status,
        }
    }

    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(self.body.as_ref().unwrap()).map_or_else(|_| None, |text| Some(text))
    }

    pub fn json<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body.as_ref().unwrap())
    }
}

#[test]
fn test_get_post_req() {
    let url = "https://github.com";
    let mut headers = List::new();
    headers.append("jar: true").unwrap();
    let client = Client::builder().default_headers(headers).build();
    let res = client.get(url).perform();
    println!("{:?}", res.text().unwrap());
}
