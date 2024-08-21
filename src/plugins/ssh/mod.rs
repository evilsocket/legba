use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};

use std::time::Duration;

use async_trait::async_trait;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

pub(crate) mod options;

super::manager::register_plugin! {
    "ssh" => SSH::new(),
    "sftp" => SSH::new()
}

#[derive(Clone)]
pub(crate) struct SSH {
    mode: options::Mode,
    passphrase: Option<String>,
}

impl SSH {
    pub fn new() -> Self {
        SSH {
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
        self.mode = opts.ssh.ssh_auth_mode.clone();
        self.passphrase.clone_from(&opts.ssh.ssh_key_passphrase);
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 22)?;
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
                address.clone(),
                &creds.username,
                method,
                ServerCheckMethod::NoCheck,
            ),
        )
        .await
        .map_err(|e| e.to_string())?;

        if res.is_ok() {
            Ok(Some(vec![Loot::new(
                "ssh",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    (key_label, creds.password.to_owned()),
                ],
            )]))
        } else if let Err(async_ssh2_tokio::Error::PasswordWrong) = res {
            Ok(None)
        } else {
            Err(res.err().unwrap().to_string())
        }
    }
}
