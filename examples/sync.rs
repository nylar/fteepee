use std::fs::File;

use fteepee_sync::{Client, Result};

fn main() -> Result<()> {
    env_logger::init();

    let mut client = Client::connect("localhost:21")?;

    client.login("username", "password")?;

    client.list("/")?;

    let mut file = File::open("Cargo.toml")?;

    client.put("Cargo.toml", &mut file)?;

    client.list("/")?;

    Ok(())
}
