use std::time::Duration;

use async_trait::async_trait;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

super::manager::register_plugin! {
    "scylla" => Scylla::new()
}

#[derive(Clone)]
pub(crate) struct Scylla {}

impl Scylla {
    pub fn new() -> Self {
        Scylla {}
    }
}

#[async_trait]
impl Plugin for Scylla {
    fn description(&self) -> &'static str {
        "ScyllaDB / Cassandra password authentication."
    }

    fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address: String = utils::parse_target_address(&creds.target, 9042)?;
        let session = scylla::SessionBuilder::new()
            .known_node(&address)
            .connection_timeout(timeout)
            .user(&creds.username, &creds.password)
            .build()
            .await;

        if session.is_ok() {
            Ok(Some(vec![Loot::new(
                "scylla",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
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
