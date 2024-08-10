use std::time::Duration;

use async_trait::async_trait;
use sqlx::pool::PoolOptions;
use sqlx::{MySql, Postgres};

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

super::manager::register_plugin! {
    "mysql" => SQL::new(Flavour::My),
    "pgsql" => SQL::new(Flavour::PG)
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) enum Flavour {
    My,
    PG,
}

impl Flavour {
    fn description(&self) -> &'static str {
        match self {
            Self::My => "MySQL password authentication.",
            Self::PG => "PostgreSQL password authentication.",
        }
    }

    fn default_port(&self) -> u16 {
        match self {
            Self::My => 3306,
            Self::PG => 5432,
        }
    }
}

#[derive(Clone)]
pub(crate) struct SQL {
    flavour: Flavour,
    port: u16,
}

impl SQL {
    pub fn new(flavour: Flavour) -> Self {
        let port = flavour.default_port();
        SQL { flavour, port }
    }

    async fn do_attempt<DB: sqlx::Database>(
        &self,
        scheme: &str,
        db: &str,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, self.port)?;
        let pool = tokio::time::timeout(
            timeout,
            PoolOptions::<DB>::new().connect(&format!(
                "{}://{}:{}@{}/{}",
                scheme, &creds.username, &creds.password, &address, db
            )),
        )
        .await
        .map_err(|e| e.to_string())?;

        if pool.is_ok() {
            Ok(Some(vec![Loot::new(
                scheme,
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

#[async_trait]
impl Plugin for SQL {
    fn description(&self) -> &'static str {
        self.flavour.description()
    }

    fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        match self.flavour {
            Flavour::My => {
                self.do_attempt::<MySql>("mysql", "mysql", creds, timeout)
                    .await
            }
            Flavour::PG => {
                self.do_attempt::<Postgres>("postgres", "postgres", creds, timeout)
                    .await
            }
        }
    }
}
