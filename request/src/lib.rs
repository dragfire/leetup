#![allow(unused_variables, unused_imports, dead_code)]
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
    cookie: Option<String>,
    referer: Option<String>,
    body: Option<Bytes>,
}

impl Request {
    pub fn new<T: AsRef<Path>>(method: Method, url: T) -> Self {
        Request {
            method,
            url: url.as_ref().to_owned(),
            headers: List::new(),
            body: None,
            referer: None,
            cookie: None,
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

    pub fn cookie(&mut self, cookie: String) {
        self.cookie = Some(cookie);
    }

    pub fn referer(&mut self, referer: String) {
        self.referer = Some(referer);
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

    pub fn referer(mut self, referer: String) -> Self {
        self.request.referer(referer);
        self
    }

    pub fn cookie(mut self, cookie: String) -> Self {
        self.request.cookie(cookie);
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
        log::debug!("Request: {:#?}", self.request);
        handle.perform(self.request)
    }
}

/// Client wraps libcurl Easy
pub struct ClientBuilder {
    headers: List,
    cookie_jar: bool,
    redirect: bool,
    http2: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        let mut headers = List::new();
        headers.append("Accept: */*").unwrap();

        ClientBuilder {
            cookie_jar: false,
            headers,
            redirect: false,
            http2: false,
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

    pub fn http2(mut self, enabled: bool) -> Self {
        self.http2 = enabled;
        self
    }

    pub fn build(mut self) -> Client {
        let mut handle = Easy::new();
        if self.cookie_jar {
            let cookie_path = "data";
            handle.cookie_jar(cookie_path).unwrap();
            self.headers.append("jar: true").unwrap();
        }

        if self.http2 {
            handle.http_version(curl::easy::HttpVersion::V2).unwrap();
        }

        log::debug!("ClientBuilder Headers: {:#?}", self.headers);
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
        Client::request(rc_client, Method::Post, url)
    }

    pub fn put<R: AsRef<Path>>(&self, url: R) -> RequestBuilder {
        let rc_client = Rc::new(self);
        Client::request(rc_client, Method::Put, url)
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

    pub fn redirect(&self, enabled: bool) -> Result<(), curl::Error> {
        self.handle.borrow_mut().follow_location(enabled)
    }

    pub fn redirect_url(&self) -> Option<String> {
        let mut handle = self.handle.borrow_mut();
        let rurl = handle.redirect_url();
        if let Ok(res) = rurl {
            res.map(String::from)
        } else {
            None
        }
    }

    pub fn url_encode<T: AsRef<[u8]>>(&self, data: T) -> String {
        self.handle.borrow_mut().url_encode(data.as_ref())
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

        if let Some(ref referer) = request.referer {
            handle.referer(referer).unwrap();
        }

        if let Some(ref cookie) = request.cookie {
            handle.cookie(cookie).unwrap();
        }

        let mut req_headers = List::new();
        for header in request.headers() {
            req_headers
                .append(std::str::from_utf8(header).unwrap())
                .unwrap();
        }

        handle.http_headers(req_headers).unwrap();

        {
            if let Some(body) = request.body() {
                handle.post_field_size(body.len() as u64).unwrap();
            }
            let mut transfer = handle.transfer();

            if request.body().is_some() {
                transfer
                    .read_function(|buf| {
                        Ok(request.body().unwrap().as_ref().read(buf).unwrap_or(0))
                    })
                    .unwrap();
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

    pub fn headers(&self) -> &Vec<String> {
        &self.headers
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

fn get_session() -> String {
    use regex::Regex;
    let url = "https://github.com/login";
    let client = Client::builder().cookie_jar(true).redirect(false).build();
    let res = client.get(url).perform();
    let text = res.text().unwrap();

    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"").unwrap();
    let auth_token = &capture_value(1, auth_token_re, text);

    let form = format!(
        "login=tom&password=thumbub&authenticity_token={}",
        client.url_encode(auth_token.as_bytes())
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

    let res = client.get(&client.redirect_url().unwrap()).perform();

    let url = "https://leetcode.com/accounts/github/login/?next=%2F";
    client.redirect(true).unwrap();
    let res = client.get(url).perform();

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
    cookie_raw
}

#[test]
fn test_get_post_req() {
    println!("{}", get_session());
}

#[test]
fn test_get_all_problems() {
    let url = "https://leetcode.com/api/problems/all";
    let client = Client::builder().redirect(true).build();
    let res = client.get(url).perform();
    println!("{:#?}", res);
    assert_eq!(200, res.status());
}

#[test]
fn test_graphql() {
    use serde_json::json;
    struct Problem {
        slug: String,
    }

    let problem = Problem {
        slug: "longest-substring-without-repeating-characters".to_string(),
    };
    let graphql = "https://leetcode.com/graphql";
    let query = r#"
    query getQuestionDetail($titleSlug: String!) {
     question(titleSlug: $titleSlug) {
       content
       stats
       likes
       dislikes
       codeDefinition
       sampleTestCase
       enableRunCode
       metaData
       translatedContent
     }
    }
    "#;
    let base = "https://leetcode.com";
    let body: serde_json::value::Value = json!({
        "query": query,
        "variables": json!({
            "titleSlug": problem.slug
        }),
        "operationName": "getQuestionDetail"
    });

    let client = Client::builder().http2(true).redirect(true).build();
    let body = body.to_string();
    let cookie = get_session();
    let cookie_header = cookie.to_string();
    let cookie = cookie.split(" ").collect::<Vec<&str>>();
    let mut csrf = cookie[0].rsplit("=").next().unwrap().to_string();
    csrf.pop();

    let res = client
        .post(graphql)
        .referer(String::from(
            "https://leetcode.com/problems/longest-substring-without-repeating-characters/",
        ))
        .cookie(cookie_header.to_string())
        .header("Host: leetcode.com")
        .header(&format!("x-csrftoken: {}", csrf))
        .header("X-Requested-With: XMLHttpRequest")
        .header("Content-Type: application/json")
        .header("Origin: https://leetcode.com")
        .body(body)
        .perform();
    let data = res.json::<serde_json::value::Value>().unwrap();
    println!("{:?}", data);
}
