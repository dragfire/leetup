use crate::{cmd::User, Result};

pub struct Session<'a> {
    pub cookie: &'a str,
}

impl<'a> Session<'a> {
    pub fn new(cookie: &'a str) -> Self {
        Session { cookie }
    }
}

pub fn login(user: User) -> Result<()> {
    println!("{:?}", user);
    Ok(())
}
