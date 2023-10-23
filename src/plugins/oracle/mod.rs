use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use sibyl as oracle;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("oracle", Box::new(Oracle::new()));
}

#[derive(Clone)]
pub(crate) struct Oracle {
    host: String,
    port: u16,
    database: String,
}

impl Oracle {
    pub fn new() -> Self {
        Oracle {
            host: String::new(),
            port: 1521,
            database: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for Oracle {
    fn description(&self) -> &'static str {
        "Oracle DB authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 1521)?;
        self.database = opts.oracle.oracle_database.clone();
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let oracle = oracle::env().map_err(|e| e.to_string())?;
        let op = tokio::time::timeout(
            timeout,
            oracle.connect(&self.database, &creds.username, &creds.password),
        )
        .await;

        if op.is_err() {
            // timeout
            Err("timed out".to_owned())
        } else if let Ok(_) = op.unwrap() {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else {
            Ok(None)
        }
    }
}
