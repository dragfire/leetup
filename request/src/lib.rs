use bytes::Bytes;
use curl::easy::{Easy, List};
use std::io::Read;
use std::path::{Path, PathBuf};

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
}

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

    pub fn method(&self) -> &Method {
        &self.method
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
    cookie_jar: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        ClientBuilder { cookie_jar: false }
    }
}

pub struct Client {
    handle: Easy,
}

impl Client {
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
}

#[test]
fn test_req() {
    let url = "https://github.com";
}
