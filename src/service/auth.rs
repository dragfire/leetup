use crate::{
    fetch,
    service::{Config, ServiceProvider},
    LeetUpError, Result,
};
use regex::Regex;
use reqwest::header;
use std::io::{BufWriter, Write};

#[derive(Debug)]
struct User {
    id: String,
    pass: String,
}

impl User {
    fn get_from_stdin() -> Self {
        return User {
            id: "dragfire".to_string(),
            pass: "d3v@github".to_string(),
        };

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

fn parse_github_request(config: &Config, res: reqwest::blocking::Response) -> Result<()> {
    let text = res.text()?;
    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"")?;
    let req_field_re = Regex::new("name=\"required_field_(.*?)\"")?;
    let ts_re = Regex::new("name=\"timestamp\" value=\"(.*?)\"")?;
    let ts_secret_re = Regex::new("name=\"timestamp_secret\" value=\"(.*?)\"")?;

    let auth_token = &capture_value(1, auth_token_re, &text)?;
    let mut req_field = capture_value(1, req_field_re, &text)?;
    let ts = &capture_value(1, ts_re, &text)?;
    let ts_secret = &capture_value(1, ts_secret_re, &text)?;
    let user = User::get_from_stdin();

    req_field = "required_field_".to_string() + &req_field;
    let form = vec![
        ("login", user.id.as_str()),
        ("password", user.pass.as_str()),
        ("authenticity_token", auth_token),
        ("commit", "Sign in"),
        ("webauthn-support", "unknown"),
        ("webauthn-iuvpaa-support", "unknown"),
        ("return_to", ""),
        (req_field.as_str(), ""),
        ("timestamp", ts),
        ("timestamp_secret", ts_secret),
    ];
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    println!("{:?}", form);

    let res = fetch::post(&config.urls.github_session_request, headers, form)?;
    println!("{:?}", res);

    Ok(())
}

pub fn github_login<'a, P: ServiceProvider<'a>>(provider: &P) -> Result<()> {
    let config = provider.config()?;
    let mut headers = header::HeaderMap::new();
    headers.insert("jar", "true".parse().unwrap());
    let client: reqwest::blocking::Client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let res = client.get(&config.urls.github_login_request).send()?;
    println!("{:?}", res);

    Ok(())
}
