use bytes::Bytes;
use curl::easy::Easy;
pub use curl::easy::List;
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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

pub struct RequestBuilder<'a> {
    client: Rc<&'a Client>,
    request: Request,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(client: Rc<&'a Client>, request: Request) -> Self {
        RequestBuilder { client, request }
    }

    pub fn header(mut self, header: &str) -> Self {
        self.request
            .headers_mut()
            .append(header)
            .expect("Unable to add header");
        self
    }

    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.request.body = Some(body.into());
        self
    }

    pub fn build(self) -> Request {
        self.request
    }

    pub fn perform(mut self) -> Response {
        let handle = Rc::get_mut(&mut self.client).unwrap();
        handle.perform(self.request)
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

    pub fn cookie_jar(mut self, enabled: bool) -> Self {
        self.cookie_jar = enabled;
        self
    }

    pub fn redirect(mut self, enabled: bool) -> Self {
        self.redirect = enabled;
        self
    }

    pub fn build(mut self) -> Client {
        let mut handle = Easy::new();
        if self.cookie_jar {
            let cookie_path = "data";
            handle.cookie_jar(cookie_path).unwrap();
            self.headers.append("jar: true").unwrap();
        }

        handle.http_headers(self.headers).unwrap();
        handle.useragent("Leetup").unwrap();
        handle.follow_location(self.redirect).unwrap();

        Client {
            handle: RefCell::new(handle),
        }
    }
}

pub struct Client {
    handle: RefCell<Easy>,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn get<R: AsRef<Path>>(&self, url: R) -> RequestBuilder {
        let rc_client = Rc::new(self);
        Client::request(rc_client, Method::Get, url)
    }

    pub fn post<R: AsRef<Path>>(&self, url: R) -> RequestBuilder {
        let rc_client = Rc::new(self);
        Client::request(rc_client, Method::Get, url)
    }

    pub fn put<R: AsRef<Path>>(&self, url: R) -> RequestBuilder {
        let rc_client = Rc::new(self);
        Client::request(rc_client, Method::Get, url)
    }

    pub fn request<R: AsRef<Path>>(
        rc_client: Rc<&Client>,
        method: Method,
        url: R,
    ) -> RequestBuilder {
        let req = Request::new(method, url);
        let client = Rc::clone(&rc_client);
        RequestBuilder::new(client, req)
    }

    pub fn cookies(&self) -> Result<List, curl::Error> {
        self.handle.borrow_mut().cookies()
    }

    pub fn perform(&self, request: Request) -> Response {
        let mut headers = Vec::new();
        let mut buf = Vec::new();
        let mut handle = self.handle.borrow_mut();
        handle.url(request.url.to_str().unwrap()).unwrap();

        match request.method() {
            Method::Get => handle.get(true).unwrap(),
            Method::Post => handle.post(true).unwrap(),
            _ => (),
        }

        {
            let mut transfer = handle.transfer();
            if let Err(e) = transfer
                .read_function(|buf| Ok(request.body().unwrap().as_ref().read(buf).unwrap_or(0)))
            {
                println!("{:?}", e);
            }
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

        let status = handle.response_code().unwrap();

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

    pub fn status(&self) -> u32 {
        self.status
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
    use regex::Regex;
    let url = "https://github.com/login";
    let client = Client::builder().cookie_jar(true).redirect(true).build();
    let res = client.get(url).perform();
    let text = res.text().unwrap();
    println!("{}", res.status());

    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"").unwrap();
    let auth_token = &capture_value(1, auth_token_re, text);

    let form = format!(
        "login=dragfire&password=d3v@github&authenticity_token={}",
        auth_token
    );

    fn capture_value(i: usize, re: Regex, text: &str) -> String {
        let caps = re.captures(text).unwrap();
        caps.get(i).map(|m| String::from(m.as_str())).unwrap()
    }

    let url = "https://github.com/session";
    let res = client
        .post(url)
        .body(form)
        .header("Content-Type: application/x-www-form-urlencoded")
        .perform();

    println!("{}", res.status());

    let url = "https://github.com";
    let res = client.get(url).perform();
    println!("{}", res.status());

    let url = "https://leetcode.com/accounts/github/login/?next=%2F";
    let res = client.get(url).perform();
    println!("{}", res.status());
    let cookies = client.cookies().unwrap();
    let mut cookie_raw = String::new();
    for cookie in cookies.iter() {
        let mut cookie = std::str::from_utf8(cookie).unwrap().rsplit("\t");
        let val = cookie.next().unwrap();
        let name = cookie.next().unwrap();
        match name {
            "LEETCODE_SESSION" => {
                cookie_raw.push_str(&format!("{}={};", "LEETCODE_SESSION", val));
            }
            "csrftoken" => cookie_raw.push_str(&format!("{}={}; ", "csrftoken", val)),
            _ => (),
        }
    }

    // remove trailing semi-colon
    cookie_raw.pop();
    println!("COOKIE: {}", cookie_raw);
}

#[test]
fn test_get_all_problems() {
    let url = "https://leetcode.com/api/problems/all";
    let client = Client::builder().redirect(true).build();
    let res = client.get(url).perform();
    assert_eq!(200, res.status());
}
