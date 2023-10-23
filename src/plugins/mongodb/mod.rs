use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use mongodb::options::Credential;

use crate::session::{Error, Loot};
use crate::Plugin;
use crate::{utils, Options};

use crate::creds::Credentials;

#[ctor]
fn register() {
    crate::plugins::manager::register("mongodb", Box::new(MongoDB::new()));
}

#[derive(Clone)]
pub(crate) struct MongoDB {
    host: String,
    port: u16,
    address: String,
}

impl MongoDB {
    pub fn new() -> Self {
        MongoDB {
            host: String::new(),
            address: String::new(),
            port: 27017,
        }
    }
}

#[async_trait]
impl Plugin for MongoDB {
    fn description(&self) -> &'static str {
        "MongoDB password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 27017)?;
        self.address = format!("{}:{}", &self.host, self.port);

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let mut opts = mongodb::options::ClientOptions::default();
        let mut cred = Credential::default();

        cred.username = Some(creds.username.to_owned());
        cred.password = Some(creds.password.to_owned());

        opts.hosts = vec![mongodb::options::ServerAddress::Tcp {
            host: self.host.to_owned(),
            port: Some(self.port),
        }];
        opts.connect_timeout = Some(timeout);
        opts.credential = Some(cred);

        let cli = mongodb::Client::with_options(opts).map_err(|e| e.to_string())?;
        let dbs = cli.list_database_names(None, None).await;

        if let Ok(dbs) = dbs {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
                ("databases".to_owned(), dbs.join(", ")),
            ])))
        } else {
            Ok(None)
        }
    }
}
