use fteepee_sync::{Client, Result};

fn main() -> Result<()> {
    let mut client = Client::connect("localhost:21")?;

    client.login("username", "password")?;

    client.list("/")?;

    Ok(())
}
