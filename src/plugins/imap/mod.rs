use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

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
pub(crate) struct IMAP {}

impl IMAP {
    pub fn new() -> Self {
        IMAP {}
    }
}

#[async_trait]
impl Plugin for IMAP {
    fn description(&self) -> &'static str {
        "IMAP password authentication."
    }

    fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let address = utils::parse_target_address(&creds.target, 993)?;
        let stream = crate::utils::net::async_tcp_stream(&address, timeout, true).await?;
        let client = async_imap::Client::new(stream);
        if client.login(&creds.username, &creds.password).await.is_ok() {
            return Ok(Some(Loot::from(
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )));
        }

        Ok(None)
    }
}
