use std::time::Duration;

use async_trait::async_trait;
use sibyl as oracle;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

pub(crate) mod options;

super::manager::register_plugin! {
    "oracle" => Oracle::new()
}

#[derive(Clone)]
pub(crate) struct Oracle {
    database: String,
}

impl Oracle {
    pub fn new() -> Self {
        Oracle {
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
        self.database = opts.oracle.oracle_database.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 1521)?;
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
            Ok(Some(vec![Loot::new(
                "oracle",
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
