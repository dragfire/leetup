use crate::{fetch, service::ServiceProvider, Result};
use reqwest::header;

pub fn github_login<'a, P: ServiceProvider<'a>>(provider: &P) -> Result<()> {
    let config = provider.config()?;
    let mut headers = header::HeaderMap::new();
    headers.insert("jar", "true".parse().unwrap());

    let res = fetch::get(&config.urls.github_login_request, headers)?;
    println!("{:#?}", res.text()?);

    Ok(())
}
