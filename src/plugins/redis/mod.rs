use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::session::{Error, Loot};
use crate::Plugin;
use crate::{utils, Options};

use crate::creds::Credentials;

pub(crate) mod options;

super::manager::register_plugin! {
    "redis" => Redis::new()
}

#[derive(Clone)]
pub(crate) struct Redis {
    ssl: bool,
}

impl Redis {
    pub fn new() -> Self {
        Redis { ssl: false }
    }
}

#[async_trait]
impl Plugin for Redis {
    fn description(&self) -> &'static str {
        "Redis legacy and ACL password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.ssl = opts.redis.redis_ssl;
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 6379)?;

        let mut stream = crate::utils::net::async_tcp_stream(&address, timeout, self.ssl).await?;

        stream
            .write_all(format!("AUTH {} {}\n", &creds.username, &creds.password).as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        let mut buffer = [0_u8; 3];

        stream
            .read_exact(&mut buffer)
            .await
            .map_err(|e| e.to_string())?;

        if buffer.starts_with(&[b'+', b'O', b'K']) {
            Ok(Some(vec![Loot::new(
                "redis",
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
