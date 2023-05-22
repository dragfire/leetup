#![allow(dead_code, unused)]
use leetup::{cmd, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    cmd::process().await?;
    Ok(())
}
