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
    command: String,
}

impl Redis {
    pub fn new() -> Self {
        Redis {
            host: String::new(),
            port: 6379,
            ssl: false,
            command: String::new(),
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
        self.command = opts.redis.redis_command.to_owned();

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let url = format!(
            "{}://{}:{}@{}:{}",
            if self.ssl { "rediss" } else { "redis" },
            &creds.username,
            &creds.password,
            &self.host,
            self.port
        );

        let client = redis::Client::open(url).map_err(|e| e.to_string())?;

        let mut conn = tokio::time::timeout(timeout, client.get_async_connection())
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        redis::cmd(&self.command)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Some(Loot::from([
            ("username".to_owned(), creds.username.to_owned()),
            ("password".to_owned(), creds.password.to_owned()),
        ])))
    }
}
