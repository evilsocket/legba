use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};

use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

pub(crate) mod options;

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
    address: String,
    mode: options::Mode,
    passphrase: Option<String>,
}

impl SSH {
    pub fn new() -> Self {
        SSH {
            host: String::new(),
            port: 22,
            address: String::new(),
            mode: options::Mode::default(),
            passphrase: None,
        }
    }
}

#[async_trait]
impl Plugin for SSH {
    fn description(&self) -> &'static str {
        "SSH/SFTP password and private key authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 22)?;
        self.mode = opts.ssh.ssh_auth_mode.clone();
        self.passphrase = opts.ssh.ssh_key_passphrase.clone();
        self.address = format!("{}:{}", &self.host, self.port);
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let (method, key_label) = match self.mode {
            options::Mode::Password => (
                AuthMethod::with_password(&creds.password),
                "password".to_owned(),
            ),
            options::Mode::Key => (
                AuthMethod::with_key_file(&creds.password, self.passphrase.as_deref()),
                "key".to_owned(),
            ),
        };

        let res = tokio::time::timeout(
            timeout,
            Client::connect(
                (self.host.clone(), self.port),
                &creds.username,
                method,
                ServerCheckMethod::NoCheck,
            ),
        )
        .await
        .map_err(|e| e.to_string())?;

        if res.is_ok() {
            Ok(Some(Loot::from(
                &self.address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    (key_label, creds.password.to_owned()),
                ],
            )))
        } else if let Err(async_ssh2_tokio::Error::PasswordWrong) = res {
            Ok(None)
        } else {
            Err(res.err().unwrap().to_string())
        }
    }
}
