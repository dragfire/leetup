use std::io::{BufWriter, Write};

use colci::Color;

use crate::{
    service::{ServiceProvider, Session},
    Result,
};

pub async fn cookie_login<'a, P: ServiceProvider<'a>>(_provider: &P) -> Result<Session> {
    let mut out = BufWriter::new(std::io::stdout());
    let stdin = std::io::stdin();

    let mut csrf = String::new();
    let mut lc_session = String::new();

    write!(out, "{}", Color::Yellow("csrftoken: ").make())?;
    out.flush()?;
    stdin.read_line(&mut csrf)?;

    write!(out, "{}", Color::Yellow("LEETCODE_SESSION: ").make())?;
    out.flush()?;
    stdin.read_line(&mut lc_session)?;

    csrf = csrf.trim().to_string();
    lc_session = lc_session.trim().to_string();

    println!("{}", Color::Green("User logged in!").make());

    Ok(Session::new(lc_session.to_string(), csrf.to_string()))
}
