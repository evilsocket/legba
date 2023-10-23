use std::time::Duration;

extern crate async_native_tls;

use async_trait::async_trait;
use ctor::ctor;
use tokio::net::TcpStream;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;

#[ctor]
fn register() {
    crate::plugins::manager::register("imap", Box::new(IMAP::new()));
}

#[derive(Clone)]
pub(crate) struct IMAP {
    host: String,
    port: u16,
}

impl IMAP {
    pub fn new() -> Self {
        IMAP {
            host: String::new(),
            port: 993,
        }
    }
}

#[async_trait]
impl Plugin for IMAP {
    fn description(&self) -> &'static str {
        "IMAP password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 993)?;
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let address = (self.host.as_ref(), self.port);
        let tcp_stream = tokio::time::timeout(timeout, TcpStream::connect(address))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        let tls = async_native_tls::TlsConnector::new()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true);

        let tls_stream = tokio::time::timeout(timeout, tls.connect(&self.host, tcp_stream))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        let client = async_imap::Client::new(tls_stream);
        if client.login(&creds.username, &creds.password).await.is_ok() {
            return Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])));
        }

        Ok(None)
    }
}
