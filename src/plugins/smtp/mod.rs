use std::time::Duration;

use async_smtp::{authentication, SmtpClient, SmtpTransport};
use async_trait::async_trait;
use tokio::io::BufStream;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;

pub(crate) mod options;

super::manager::register_plugin! {
    "smtp" => SMTP::new()
}

#[derive(Clone)]
pub(crate) struct SMTP {
    mechanism: authentication::Mechanism,
}

impl SMTP {
    pub fn new() -> Self {
        SMTP {
            mechanism: authentication::Mechanism::Plain,
        }
    }
}

#[async_trait]
impl Plugin for SMTP {
    fn description(&self) -> &'static str {
        "SMTP password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.mechanism = match opts.smtp.smtp_mechanism.as_ref() {
            "PLAIN" => authentication::Mechanism::Plain,
            "LOGIN" => authentication::Mechanism::Login,
            "XOAUTH2" => authentication::Mechanism::Xoauth2,
            _ => {
                return Err(format!("'{}' is not a valid authentication mechanism, only PLAIN., LOGIN or XOAUTH2 are accepted.", &opts.smtp.smtp_mechanism));
            }
        };

        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 25)?;
        let stream = crate::utils::net::async_tcp_stream(&address, timeout, false).await?;

        let client = SmtpClient::new();
        let mut transport =
            tokio::time::timeout(timeout, SmtpTransport::new(client, BufStream::new(stream)))
                .await
                .map_err(|e: tokio::time::error::Elapsed| e.to_string())?
                .map_err(|e| e.to_string())?;

        let credentials =
            authentication::Credentials::new(creds.username.clone(), creds.password.clone());

        if transport.auth(self.mechanism, &credentials).await.is_ok() {
            Ok(Some(vec![Loot::new(
                "smtp",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
        } else {
            Ok(None)
        }
    }
}
