use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};

use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

#[ctor]
fn register() {
    let ssh = Box::new(SSH::new());
    crate::plugins::manager::register("ssh", ssh.clone());
    crate::plugins::manager::register("sftp", ssh);
}

#[derive(Clone)]
pub(crate) struct SSH {
    host: String,
    port: u16,
}

impl SSH {
    pub fn new() -> Self {
        SSH {
            host: String::new(),
            port: 22,
        }
    }
}

#[async_trait]
impl Plugin for SSH {
    fn description(&self) -> &'static str {
        "SSH/SFTP password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 22)?;
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let res = tokio::time::timeout(
            timeout,
            Client::connect(
                (self.host.clone(), self.port),
                &creds.username,
                AuthMethod::with_password(&creds.password),
                ServerCheckMethod::NoCheck,
            ),
        )
        .await
        .map_err(|e| e.to_string())?;

        if res.is_ok() {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else if let Err(async_ssh2_tokio::Error::PasswordWrong) = res {
            Ok(None)
        } else {
            Err(res.err().unwrap().to_string())
        }
    }
}
