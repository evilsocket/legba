use std::time::Duration;

use async_trait::async_trait;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::pool::PoolOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::{MySql, Postgres};

use crate::Options;
use crate::Plugin;
use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;

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

    async fn do_attempt<DB>(
        &self,
        scheme: &str,
        creds: &Credentials,
        timeout: Duration,
        connect_options: <<DB as sqlx::Database>::Connection as sqlx::Connection>::Options,
    ) -> Result<Option<Vec<Loot>>, Error>
    where
        DB: sqlx::Database,
    {
        let address = utils::parse_target_address(&creds.target, self.port)?;
        let pool_result = tokio::time::timeout(
            timeout,
            PoolOptions::<DB>::new().connect_with(connect_options),
        )
        .await;

        match pool_result {
            Ok(Ok(_pool)) => {
                // Connection fully successful
                Ok(Some(vec![Loot::new(
                    scheme,
                    &address,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )]))
            }
            Ok(Err(e)) => {
                let error_msg = e.to_string();
                
                // Check if authentication succeeded but database access was denied
                if self.flavour == Flavour::My {
                    // MySQL: Correct password but no database access permission
                    if error_msg.contains("Access denied") && error_msg.contains("to database") {
                        return Ok(Some(vec![Loot::new(
                            scheme,
                            &address,
                            [
                                ("username".to_owned(), creds.username.to_owned()),
                                ("password".to_owned(), creds.password.to_owned()),
                            ],
                        )]));
                    }
                } else if self.flavour == Flavour::PG {
                    // PostgreSQL: Similar permission error check
                    if error_msg.contains("permission denied for database") {
                        return Ok(Some(vec![Loot::new(
                            scheme,
                            &address,
                            [
                                ("username".to_owned(), creds.username.to_owned()),
                                ("password".to_owned(), creds.password.to_owned()),
                            ],
                        )]));
                    }
                }
                
                // Other errors (including incorrect password)
                Ok(None)
            }
            Err(_) => {
                // Timeout error
                Err("Connection timeout".into())
            }
        }
    }
}

#[async_trait]
impl Plugin for SQL {
    fn description(&self) -> &'static str {
        self.flavour.description()
    }

    async fn setup(&mut self, _opts: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let (host, port) = utils::parse_target(&creds.target, self.port)?;
        match self.flavour {
            Flavour::My => {
                let opts = MySqlConnectOptions::new()
                    .host(&host)
                    .port(port)
                    .username(&creds.username)
                    .password(&creds.password)
                    .database("mysql");
                self.do_attempt::<MySql>("mysql", creds, timeout, opts)
                    .await
            }
            Flavour::PG => {
                let opts = PgConnectOptions::new()
                    .host(&host)
                    .port(port)
                    .username(&creds.username)
                    .password(&creds.password)
                    .database("postgres");
                self.do_attempt::<Postgres>("postgres", creds, timeout, opts)
                    .await
            }
        }
    }
}