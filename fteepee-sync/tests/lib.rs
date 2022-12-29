use std::collections::HashMap;

use fteepee_sync::Client;
use testcontainers::{core::WaitFor, Image};

#[test]
fn integration_test_successful_login() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(FtpImage::default());

    let addr = format!("0.0.0.0:{}", node.get_host_port_ipv4(21));

    let mut client = Client::connect(addr).unwrap();

    client.login("user", "pass").unwrap();
}

#[test]
#[should_panic]
fn integration_test_unsuccessful_login() {
    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(FtpImage::default());

    let addr = format!("0.0.0.0:{}", node.get_host_port_ipv4(21));

    let mut client = Client::connect(addr).unwrap();

    client.login("anonymous", "coward").unwrap();
}

#[derive(Debug)]
pub struct FtpImage {
    env_vars: HashMap<String, String>,
}

impl Default for FtpImage {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("FTP_USER_NAME".to_owned(), "user".to_owned());
        env_vars.insert("FTP_USER_PASS".to_owned(), "pass".to_owned());
        env_vars.insert("FTP_USER_HOME".to_owned(), "/home/user".to_owned());

        Self { env_vars }
    }
}

impl Image for FtpImage {
    type Args = ();

    fn name(&self) -> String {
        "stilliard/pure-ftpd".to_owned()
    }

    fn tag(&self) -> String {
        "latest".to_owned()
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::StdOutMessage {
            message: "Starting Pure-FTPd:".to_owned(),
        }]
    }
}
