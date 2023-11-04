use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use lazy_static::lazy_static;
use sqlx::pool::PoolOptions;
use sqlx::{MySql, Postgres};

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

lazy_static! {
    static ref DESCRIPTIONS: HashMap<Flavour, &'static str> = {
        HashMap::from([
            (Flavour::My, "MySQL password authentication."),
            (Flavour::PG, "PostgreSQL password authentication."),
        ])
    };
    static ref DEFAULT_PORTS: HashMap<Flavour, u16> =
        HashMap::from([(Flavour::My, 3306), (Flavour::PG, 5432),]);
}

#[ctor]
fn register() {
    crate::plugins::manager::register("mysql", Box::new(SQL::new(Flavour::My)));
    crate::plugins::manager::register("pgsql", Box::new(SQL::new(Flavour::PG)));
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) enum Flavour {
    My,
    PG,
}

#[derive(Clone)]
pub(crate) struct SQL {
    flavour: Flavour,
    port: u16,
}

impl SQL {
    pub fn new(flavour: Flavour) -> Self {
        let port = *DEFAULT_PORTS.get(&flavour).unwrap();
        SQL { flavour, port }
    }

    async fn do_attempt<DB: sqlx::Database>(
        &self,
        scheme: &str,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Loot>, Error> {
        let address = utils::parse_target_address(&creds.target, self.port)?;
        let pool = tokio::time::timeout(
            timeout,
            PoolOptions::<DB>::new().connect(&format!(
                "{}://{}:{}@{}/",
                scheme, &creds.username, &creds.password, &address
            )),
        )
        .await
        .map_err(|e| e.to_string())?;

        if pool.is_ok() {
            Ok(Some(Loot::from(
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Plugin for SQL {
    fn description(&self) -> &'static str {
        DESCRIPTIONS.get(&self.flavour).unwrap()
    }

    fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        match self.flavour {
            Flavour::My => self.do_attempt::<MySql>("mysql", creds, timeout).await,
            Flavour::PG => {
                self.do_attempt::<Postgres>("postgres", creds, timeout)
                    .await
            }
        }
    }
}
