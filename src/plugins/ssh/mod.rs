use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use russh::client::GexParams;
use russh::SshId;

use std::borrow::Cow;
use std::time::Duration;

use async_trait::async_trait;
use russh::kex::ALL_KEX_ALGORITHMS;

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

    async fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.mode = opts.ssh.ssh_auth_mode.clone();
        self.passphrase.clone_from(&opts.ssh.ssh_key_passphrase);
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        log::debug!("ssh key file: {}", &creds.password);
        let address = utils::parse_target_address(&creds.target, 22)?;
        let (method, key_label) = match self.mode {
            options::Mode::Password => (
                AuthMethod::with_password(&creds.password),
                "password".to_owned(),
            ),
            options::Mode::Key => (
                AuthMethod::with_key_file(
                    creds.password.strip_prefix('@').unwrap_or(&creds.password),
                    self.passphrase.as_deref()
                ),
                "key".to_owned(),
            ),
        };

        let mut config = async_ssh2_tokio::Config::default();

        // se all available key exchange algorithms for maximum compatibility
        // https://github.com/evilsocket/legba/issues/71
        config.preferred.kex = Cow::Owned(ALL_KEX_ALGORITHMS.iter().map(|&n| *n).collect());
        // The Diffie-Hellman group used in diffie-hellman-group14-sha1 has a group size of 2048 bits. 
        config.gex = GexParams::new(2048, config.gex.preferred_group_size(), config.gex.max_group_size()).unwrap();
        // set the client id to a less fingerprintable value
        config.client_id = SshId::Standard("SSH-2.0-OpenSSH_9.8".to_string());

        let res = tokio::time::timeout(
            timeout,
            Client::connect_with_config(
                address.clone(),
                &creds.username,
                method,
                ServerCheckMethod::NoCheck,
                config,
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
            log::debug!("password wrong");
            Ok(None)
        } else {
            log::info!("error: {:?}", &res);
            Err(res.err().unwrap().to_string())
        }
    }
}
