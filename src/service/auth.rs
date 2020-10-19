use crate::{
    service::{ServiceProvider, Session},
    LeetUpError, Result,
};
use colci::Color;
use regex::Regex;
use std::io::{BufWriter, Write};
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

        write!(out, "{}", Color::Yellow("Username: ").make()).ok();
        out.flush().unwrap();
        stdin.read_line(&mut id).unwrap();
        id = id.trim().to_string();

        let pass =
            rpassword::prompt_password_stdout(Color::Yellow("Password: ").make().as_str()).unwrap();

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
    let client = request::Client::builder()
        .cookie_jar(true)
        .redirect(false)
        .build();
    let res = client.get(&config.urls.github_login_request).perform();
    let text = res.text().unwrap();

    let auth_token_re = Regex::new("name=\"authenticity_token\" value=\"(.*?)\"").unwrap();
    let auth_token = &capture_value(1, auth_token_re, text)?;
    let user = User::get_from_stdin();

    let form = format!(
        "login={}&password={}&authenticity_token={}",
        user.id, user.pass, auth_token
    );

    let _res = client
        .post(&config.urls.github_session_request)
        .body(form)
        .header("Content-Type: application/x-www-form-urlencoded")
        .perform();

    let redirect_url = client.redirect_url();
    if let Some(ref url) = redirect_url {
        let _res = client.get(url).perform();
    }

    client.redirect(true).unwrap();

    let _res = client.get(&config.urls.github_login).perform();
    if _res.status() != 200 {
        eprintln!("{:#?}", _res);
    }

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

    let session = Session::from_str(&cookie_raw).unwrap();

    Ok(session)
}
