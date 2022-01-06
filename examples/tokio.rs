use fteepee_tokio::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("localhost:21").await?;

    client.login("username", "password").await?;

    client.list("/").await?;

    Ok(())
}
