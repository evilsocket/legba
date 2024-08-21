use std::time::Duration;

use async_trait::async_trait;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;

pub(crate) mod options;

super::manager::register_plugin! {
    "pop3" => POP3::new()
}

#[derive(Clone)]
pub(crate) struct POP3 {
    ssl: bool,
}

impl POP3 {
    pub fn new() -> Self {
        POP3 { ssl: false }
    }
}

#[async_trait]
impl Plugin for POP3 {
    fn description(&self) -> &'static str {
        "POP3 password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.ssl = opts.pop3.pop3_ssl;
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let (host, port) = utils::parse_target(&creds.target, 110)?;
        let address = (host, port);

        if self.ssl {
            let tls = async_native_tls::TlsConnector::new()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);

            let mut client = tokio::time::timeout(timeout, async_pop::connect(&address, "", &tls))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

            if client.login(&creds.username, &creds.password).await.is_ok() {
                return Ok(Some(vec![Loot::new(
                    "pop3",
                    &address.0,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )]));
            }
        } else {
            let mut client = tokio::time::timeout(timeout, async_pop::connect_plain(&address))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

            if client.login(&creds.username, &creds.password).await.is_ok() {
                return Ok(Some(vec![Loot::new(
                    "pop3",
                    &address.0,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )]));
            }
        }

        Ok(None)
    }
}
