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
    crate::plugins::manager::register("scylla", Box::new(Scylla::new()));
}

#[derive(Clone)]
pub(crate) struct Scylla {
    host: String,
    port: u16,
    address: String,
}

impl Scylla {
    pub fn new() -> Self {
        Scylla {
            host: String::new(),
            port: 9042,
            address: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for Scylla {
    fn description(&self) -> &'static str {
        "ScyllaDB / Cassandra password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 9042)?;
        self.address = format!("{}:{}", &self.host, self.port);
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let session = scylla::SessionBuilder::new()
            .known_node(&self.address)
            .connection_timeout(timeout)
            .user(&creds.username, &creds.password)
            .build()
            .await;

        if session.is_ok() {
            Ok(Some(Loot::from(
                &self.address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )))
        } else {
            // this client library doesn't differentiate between a connection error and bad credentials
            let err = session.err().unwrap().to_string();
            if err.contains("Authentication failed") {
                Ok(None)
            } else {
                Err(err)
            }
        }
    }
}
