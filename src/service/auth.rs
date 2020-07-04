use crate::{
    service::{ServiceProvider, Session},
    LeetUpError, Result,
};
use curl::easy::{Easy, List};
use regex::Regex;
use std::fs;
use std::io::{BufWriter, Read, Write};
use std::str::FromStr;

#[derive(Debug)]
struct User {
    id: String,
    pass: String,
}

impl User {
    fn get_from_stdin() -> Self {
        let mut out = BufWriter::new(std::io::stdout());
        let stdin = std::io::stdin();
        let mut id = String::new();

        write!(out, "Username: ").ok();
        out.flush().unwrap();
        stdin.read_line(&mut id).unwrap();
        id = id.trim().to_string();

        let pass = rpassword::prompt_password_stdout("Password: ").unwrap();

        User { id, pass }
    }
}

fn capture_value(i: usize, re: Regex, text: &str) -> Result<String> {
    let caps = re.captures(text).ok_or(LeetUpError::OptNone)?;
    caps.get(i)
        .map(|m| String::from(m.as_str()))
        .ok_or(LeetUpError::OptNone)
}

pub fn github_login<'a, P: ServiceProvider<'a>>(provider: &P) -> Result<Session> {
    let config = provider.config()?;
    let mut handle = Easy::new();

    let cookie_path = "data";
    fs::create_dir_all(cookie_path).unwrap();

    handle.cookie_jar(cookie_path).unwrap();
    let mut list = List::new();
    list.append("jar: true").unwrap();
    handle.http_headers(list).unwrap();

    // get auth_token from github
    let mut buf = Vec::new();
    handle.url(&config.urls.github_login_request).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let text = std::str::from_utf8(&buf).unwrap();
    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"")?;
    let auth_token = &capture_value(1, auth_token_re, text)?;
    let auth_token = handle.url_encode(auth_token.as_bytes());
    let user = User::get_from_stdin();

    let form = format!(
        "login={}&password={}&authenticity_token={}",
        user.id, user.pass, auth_token
    );
    println!("{}", form);
    let mut form = form.as_bytes();

    handle.url(&config.urls.github_session_request).unwrap();
    let mut list = List::new();
    list.append("Content-Type: application/x-www-form-urlencoded")
        .unwrap();
    handle.http_headers(list).unwrap();
    handle.post(true).unwrap();
    handle.post_field_size(form.len() as u64).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .read_function(|buf| Ok(form.read(buf).unwrap_or(0)))
            .unwrap();
        transfer.perform().unwrap();
    }

    let redirect_url = handle.redirect_url();
    let redirect_url = redirect_url.unwrap().unwrap().to_string();
    handle.url(&redirect_url).unwrap();
    handle.get(true).unwrap();
    handle.perform().unwrap();

    let mut headers = Vec::new();
    handle.url(&config.urls.github_login).unwrap();
    handle.follow_location(true).unwrap();
    handle.get(true).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .header_function(|header| {
                headers.push(std::str::from_utf8(header).unwrap().to_string());
                true
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let cookies = handle.cookies().unwrap();
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

    let session = Session::from_str(&cookie_raw).unwrap();

    Ok(session)
}
