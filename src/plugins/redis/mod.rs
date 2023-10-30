use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::session::{Error, Loot};
use crate::Plugin;
use crate::{utils, Options};

use crate::creds::Credentials;
pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("redis", Box::new(Redis::new()));
}

#[derive(Clone)]
pub(crate) struct Redis {
    host: String,
    port: u16,
    ssl: bool,
    address: String,
}

impl Redis {
    pub fn new() -> Self {
        Redis {
            host: String::new(),
            port: 6379,
            ssl: false,
            address: String::new(),
        }
    }

    async fn attempt_with_stream<S>(
        &self,
        creds: &Credentials,
        mut stream: S,
    ) -> Result<Option<Loot>, Error>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let auth = format!("AUTH {} {}\n", &creds.username, &creds.password);

        stream
            .write_all(auth.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        let mut buffer = [0_u8; 128];

        stream.read(&mut buffer).await.map_err(|e| e.to_string())?;

        if buffer.starts_with(&[b'+', b'O', b'K']) {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Plugin for Redis {
    fn description(&self) -> &'static str {
        "Redis legacy and ACL password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 6379)?;
        self.ssl = opts.redis.redis_ssl;
        self.address = format!("{}:{}", &self.host, self.port);

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let tcp_stream = tokio::time::timeout(timeout, TcpStream::connect(&self.address))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        if !self.ssl {
            self.attempt_with_stream(creds, tcp_stream).await
        } else {
            let tls = async_native_tls::TlsConnector::new()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);

            let stream = tokio::time::timeout(timeout, tls.connect(&self.host, tcp_stream))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

            self.attempt_with_stream(creds, stream).await
        }
    }
}
