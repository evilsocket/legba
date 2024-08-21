use async_ftp::FtpStream;

use std::time::Duration;

use async_trait::async_trait;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

super::manager::register_plugin! {
    "ftp" => FTP::new()
}

#[derive(Clone)]
pub(crate) struct FTP {}

impl FTP {
    pub fn new() -> Self {
        FTP {}
    }
}

#[async_trait]
impl Plugin for FTP {
    fn description(&self) -> &'static str {
        "FTP password authentication."
    }

    fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 21)?;

        let mut stream = tokio::time::timeout(timeout, FtpStream::connect(&address))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        if stream.login(&creds.username, &creds.password).await.is_ok() {
            Ok(Some(vec![Loot::new(
                "ftp",
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
