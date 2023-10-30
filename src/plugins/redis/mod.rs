use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

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
}

impl Redis {
    pub fn new() -> Self {
        Redis {
            host: String::new(),
            port: 6379,
            ssl: false,
        }
    }
}

#[async_trait]
impl Plugin for Redis {
    fn description(&self) -> &'static str {
        "Redis ACL password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 6379)?;
        self.ssl = opts.redis.redis_ssl;

        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        _timeout: Duration,
    ) -> Result<Option<Loot>, Error> {
        let url = format!(
            "{}://{}:{}@{}:{}",
            if self.ssl { "rediss" } else { "redis" },
            &creds.username,
            &creds.password,
            &self.host,
            self.port
        );

        let client = redis::Client::open(url).map_err(|e| e.to_string())?;
        let mut conn = client
            .get_async_connection() // there is no get_async_connection_with_timeout() method
            .await
            .map_err(|e| e.to_string())?;

        redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Some(Loot::from([
            ("username".to_owned(), creds.username.to_owned()),
            ("password".to_owned(), creds.password.to_owned()),
        ])))
    }
}
