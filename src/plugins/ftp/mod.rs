use async_ftp::FtpStream;

use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

#[ctor]
fn register() {
    crate::plugins::manager::register("ftp", Box::new(FTP::new()));
}

#[derive(Clone)]
pub(crate) struct FTP {
    host: String,
    port: u16,
    address: String,
}

impl FTP {
    pub fn new() -> Self {
        FTP {
            host: String::new(),
            port: 21,
            address: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for FTP {
    fn description(&self) -> &'static str {
        "FTP password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 21)?;
        self.address = format!("{}:{}", &self.host, self.port);
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let mut stream = tokio::time::timeout(timeout, FtpStream::connect(&self.address))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        if stream.login(&creds.username, &creds.password).await.is_ok() {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else {
            Ok(None)
        }
    }
}
