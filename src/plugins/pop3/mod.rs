use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("pop3", Box::new(POP3::new()));
}

#[derive(Clone)]
pub(crate) struct POP3 {
    host: String,
    port: u16,
    address: String,
    ssl: bool,
}

impl POP3 {
    pub fn new() -> Self {
        POP3 {
            host: String::new(),
            port: 110,
            address: String::new(),
            ssl: false,
        }
    }
}

#[async_trait]
impl Plugin for POP3 {
    fn description(&self) -> &'static str {
        "POP3 password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 110)?;
        self.ssl = opts.pop3.pop3_ssl;
        self.address = format!("{}:{}", &self.host, self.port);
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let address = (self.host.as_ref(), self.port);

        if self.ssl {
            let tls = async_native_tls::TlsConnector::new()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);

            let mut client =
                tokio::time::timeout(timeout, async_pop::connect(address, &self.host, &tls))
                    .await
                    .map_err(|e| e.to_string())?
                    .map_err(|e| e.to_string())?;

            if client.login(&creds.username, &creds.password).await.is_ok() {
                return Ok(Some(Loot::from(
                    &self.address,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )));
            }
        } else {
            let mut client = tokio::time::timeout(timeout, async_pop::connect_plain(address))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

            if client.login(&creds.username, &creds.password).await.is_ok() {
                return Ok(Some(Loot::from(
                    &self.address,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )));
            }
        }

        Ok(None)
    }
}
