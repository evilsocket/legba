use std::time::Duration;

use async_trait::async_trait;
use mongodb::options::Credential;

use crate::session::{Error, Loot};
use crate::Plugin;
use crate::{utils, Options};

use crate::creds::Credentials;

super::manager::register_plugin! {
    "mongodb" => MongoDB::new()
}

#[derive(Clone)]
pub(crate) struct MongoDB {}

impl MongoDB {
    pub fn new() -> Self {
        MongoDB {}
    }
}

#[async_trait]
impl Plugin for MongoDB {
    fn description(&self) -> &'static str {
        "MongoDB password authentication."
    }

    fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let (host, port) = utils::parse_target(&creds.target, 27017)?;

        let mut opts = mongodb::options::ClientOptions::default();
        let mut cred = Credential::default();

        cred.username = Some(creds.username.to_owned());
        cred.password = Some(creds.password.to_owned());

        opts.hosts = vec![mongodb::options::ServerAddress::Tcp {
            host: host.to_owned(),
            port: Some(port),
        }];
        opts.connect_timeout = Some(timeout);
        opts.credential = Some(cred);

        let cli = mongodb::Client::with_options(opts).map_err(|e| e.to_string())?;
        let dbs = cli.list_database_names(None, None).await;

        if let Ok(dbs) = dbs {
            Ok(Some(vec![Loot::new(
                "mongodb",
                &host,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                    ("databases".to_owned(), dbs.join(", ")),
                ],
            )]))
        } else {
            Ok(None)
        }
    }
}
