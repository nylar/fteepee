use fteepee_tokio::{Client, Result};
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("localhost:21").await?;

    client.login("username", "password").await?;

    client.list("/").await?;

    let mut file = File::open("Cargo.toml").await?;

    client.put("Cargo.toml", &mut file).await?;

    client.list("/").await?;

    Ok(())
}
