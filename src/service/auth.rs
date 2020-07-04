use crate::{
    fetch,
    service::{Config, ServiceProvider},
    LeetUpError, Result,
};
use regex::Regex;
use reqwest::{
    blocking::{self, Client, Request},
    header, redirect,
};
use std::borrow::Borrow;
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

pub fn github_login<'a, P: ServiceProvider<'a>>(provider: &P) -> Result<()> {
    let config = provider.config()?;
    let custom = redirect::Policy::custom(|attempt| {
        eprintln!("{}, Location: {:?}", attempt.status(), attempt.url());
        redirect::Policy::default().redirect(attempt)
    });
    let client: Client = Client::builder()
        .redirect(custom)
        .cookie_store(true)
        .referer(true)
        .build()?;

    let res = client.get(&config.urls.github_login_request).send()?;
    let text = res.text()?;

    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"")?;
    let auth_token = &capture_value(1, auth_token_re, &text)?;
    let user = User::get_from_stdin();

    let form = &[
        ("login", "dragfire"),
        ("password", "d3v@github"),
        ("authenticity_token", auth_token),
    ];

    client
        .post(&config.urls.github_session_request)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(form)
        .send()?;

    let req = client.get(&config.urls.github_login).build()?;
    println!("{:?}", req.headers());
    let res = client.execute(req)?;
    println!("{:?}", res.cookies().collect::<Vec<_>>());

    Ok(())
}
