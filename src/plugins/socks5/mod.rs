use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("socks5", Box::new(Socks5::new()));
}

#[derive(Clone)]
pub(crate) struct Socks5 {
    remote_address: String,
    remote_port: u16,
}

impl Socks5 {
    pub fn new() -> Self {
        Socks5 {
            remote_address: "ifcfg.co".to_owned(),
            remote_port: 80,
        }
    }
}

#[async_trait]
impl Plugin for Socks5 {
    fn description(&self) -> &'static str {
        "SOCKS5 password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.remote_address = opts.socks5.socks5_address.clone();
        self.remote_port = opts.socks5.socks5_port;

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let address: String = utils::parse_target_address(&creds.target, 1080)?;
        let res = tokio::time::timeout(
            timeout,
            fast_socks5::client::Socks5Stream::connect_with_password(
                address.clone(),
                self.remote_address.clone(),
                self.remote_port,
                creds.username.clone(),
                creds.password.clone(),
                fast_socks5::client::Config::default(),
            ),
        )
        .await
        .map_err(|e| e.to_string())?;

        return Ok(if res.is_ok() {
            Some(Loot::new(
                "socks5",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            ))
        } else {
            None
        });
    }
}
