use fteepee_tokio::{Client, Result};
use tokio::fs::File;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let mut client = Client::connect("localhost:21").await?;

    client.login("username", "password").await?;

    client.list("/").await?;

    let mut file = File::open("Cargo.toml").await?;

    client.put("Cargo.toml", &mut file).await?;

    client.list("/").await?;

    Ok(())
}
