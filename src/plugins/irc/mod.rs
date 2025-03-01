use rand::Rng;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use async_trait::async_trait;

use crate::Options;
use crate::Plugin;
use crate::session::{Error, Loot};
use crate::utils;

pub(crate) mod options;

use crate::creds::Credentials;

super::manager::register_plugin! {
    "irc" => IRC::new()
}

#[derive(Clone)]
pub(crate) struct IRC {
    tls: bool,
}

impl IRC {
    pub fn new() -> Self {
        IRC { tls: false }
    }

    fn generate_random_username() -> String {
        let mut rng = rand::rng();
        let length = rng.random_range(5..=9);
        (0..length)
            .map(|_| {
                let range = if rng.random_bool(0.5) {
                    b'a'..=b'z'
                } else {
                    b'A'..=b'Z'
                };
                rng.random_range(range) as char
            })
            .collect()
    }
}

#[async_trait]
impl Plugin for IRC {
    fn description(&self) -> &'static str {
        "IRC server password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.tls = opts.irc.irc_tls;
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address =
            utils::parse_target_address(&creds.target, if self.tls { 6697 } else { 6667 })?;

        let mut stream = crate::utils::net::async_tcp_stream(&address, timeout, self.tls).await?;

        let username = IRC::generate_random_username();
        stream
            .write_all(
                format!(
                    "NICK {}\r\nUSER {} 0 * :{}\r\nPASS {}\r\n",
                    username, username, username, creds.password
                )
                .as_bytes(),
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut buffer = vec![0; 1024];
        let mut accumulated_data = Vec::new();
        loop {
            let bytes_read = stream.read(&mut buffer).await.map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                return Ok(None);
            }
            accumulated_data.extend_from_slice(&buffer[..bytes_read]);
            let response = String::from_utf8_lossy(&accumulated_data);
            if response.contains(" 001 ") && response.contains("Welcome") {
                return Ok(Some(vec![Loot::new(
                    "irc",
                    &address,
                    [("password".to_owned(), creds.password.to_owned())],
                )]));
            }
        }
    }
}
