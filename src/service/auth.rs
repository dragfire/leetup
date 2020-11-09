use crate::{
    service::{ServiceProvider, Session},
    LeetUpError, Result,
};
use colci::Color;
use log::{debug, error, info};
use regex::Regex;
use reqwest::{
    blocking::{Body, Client, RequestBuilder, Response},
    header::HeaderMap,
    header::HeaderValue,
    header::CONTENT_TYPE,
    header::COOKIE,
    header::SET_COOKIE,
    StatusCode,
};
use spinners::{Spinner, Spinners};
use std::io::{BufWriter, Write};
use std::str::FromStr;

#[derive(Debug)]
struct User {
    id: String,
    pass: String,
}

impl User {
    fn get_from_stdin() -> Result<Self> {
        let mut out = BufWriter::new(std::io::stdout());
        let stdin = std::io::stdin();
        let mut id = String::new();

        write!(out, "{}", Color::Yellow("Username: ").make()).ok();
        out.flush().unwrap();
        stdin.read_line(&mut id).unwrap();
        id = id.trim().to_string();

        let pass =
            rpassword::prompt_password_stdout(Color::Yellow("Password: ").make().as_str()).unwrap();

        if id.len() == 0 || pass.len() == 0 {
            Err(LeetUpError::Any(anyhow::anyhow!(
                "Username or password is empty"
            )))
        } else {
            Ok(User { id, pass })
        }
    }
}

fn capture_value(i: usize, re: Regex, text: &str) -> Result<String> {
    let caps = re.captures(text).ok_or(LeetUpError::OptNone)?;
    caps.get(i)
        .map(|m| String::from(m.as_str()))
        .ok_or(LeetUpError::OptNone)
}

pub fn github_login<'a, P: ServiceProvider>(provider: &P) -> Result<Session> {
    let client_err = LeetUpError::Any(anyhow::anyhow!("Something went wrong!"));
    let config = provider.config()?.ok_or(LeetUpError::OptNone)?;
    let client = Client::builder()
        .cookie_store(true)
        .referer(true)
        .redirect(reqwest::redirect::Policy::none()) // disable it: CookieStore does not play well with redirects.
        .build()?;
    let res = client.get(&config.urls.github_login_request).send()?;
    if res.status() != 200 {
        error!("Status: {}", res.status());
        return Err(client_err);
    }
    let text = &res.text()?;

    let ga_id_re = Regex::new("name=\"ga_id\" value=\"(.*?)\"").unwrap();
    let ga_id = capture_value(1, ga_id_re, text).unwrap_or("".to_string());
    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"").unwrap();
    let auth_token = capture_value(1, auth_token_re, text).unwrap();
    let req_field_re = Regex::new("name=\"required_field_(.*?)\"").unwrap();
    let req_field = &format!("required_field_{}", &capture_value(1, req_field_re, text)?);
    let timestamp_re = Regex::new("name=\"timestamp\" value=\"(.*?)\"").unwrap();
    let timestamp = capture_value(1, timestamp_re, text)?;
    let timestamp_secret_re = Regex::new("name=\"timestamp_secret\" value=\"(.*?)\"").unwrap();
    let timestamp_secret = capture_value(1, timestamp_secret_re, text)?;
    let user = User::get_from_stdin()?;
    let sp = Spinner::new(Spinners::Dots9, "Logging in...".into());

    let form = &[
        ("login", user.id),
        ("password", user.pass),
        ("authenticity_token", auth_token),
        ("commit", "Sign+In".to_string()),
        ("ga_id", ga_id),
        ("webauthn-support", "supported".to_string()),
        ("webauthn-iuvpaa-support", "unsupported".to_string()),
        ("return_to", "".to_string()),
        ("timestamp", timestamp),
        ("timestamp_secret", timestamp_secret),
        (req_field, "".to_string()),
    ];

    let mut res = client
        .post(&config.urls.github_session_request)
        .form(form)
        .send()?;

    info!("Status: {}", res.status());
    if res.status() == 302 {
        info!("Redirecting to: {}", res.url().as_str());
        client.get(res.url().as_str()).send()?;
    } else {
        return Err(client_err);
    }
    debug!("Headers: {:#?}", res.headers());

    res = client.get(&config.urls.github_login).send()?;
    let mut session: Session = Default::default();
    if res.status() == 302 {
        // Due to some limitations with `reqwest`, I had to manually handle
        // a few redirects to get `LEETCODE_SESSION` and `CSRFToken`
        // Issue: https://github.com/seanmonstar/reqwest/issues/14
        //
        // Eventhough it supports CookieStore, it's not quite flexible to make
        // it work the way I want it to be for LeetUp.
        info!("Redirecting to: {}", res.url().as_str());
        res = client.get(res.url().as_str()).send()?;
        info!("Status: {}", res.status());
        if !is_ok_or_redirect(res.status()) {
            return Err(client_err);
        }
        let location = res.headers().get("location").unwrap().to_str().unwrap();
        info!("Redirecting to: {}", location);
        res = client.get(location).send()?;
        info!("Status: {}", res.status());
        if !is_ok_or_redirect(res.status()) {
            return Err(client_err);
        }

        let location = res.headers().get("location").unwrap().to_str().unwrap();
        info!("Redirecting to: {}", location);
        res = client.get(location).send()?;
        info!("Status: {}", res.status());
        if !is_ok_or_redirect(res.status()) {
            return Err(client_err);
        }

        let cookies = res.cookies().collect::<Vec<_>>();
        let mut cookie_raw = String::new();
        for cookie in cookies.iter() {
            let val = cookie.value();
            let name = cookie.name();
            match name {
                "LEETCODE_SESSION" => {
                    cookie_raw.push_str(&format!("{}={};", "LEETCODE_SESSION", val));
                }
                "csrftoken" => cookie_raw.push_str(&format!("{}={}; ", "csrftoken", val)),
                _ => (),
            }
        }

        session = Session::from_str(&cookie_raw).unwrap();
        info!("Session: {:#?}", session);
    } else if res.status() != 200 {
        error!("Status: {}", res.status());
        return Err(client_err);
    }
    sp.stop();

    Ok(session)
}

fn is_ok_or_redirect(status: reqwest::StatusCode) -> bool {
    status == 200 || status == 302
}
