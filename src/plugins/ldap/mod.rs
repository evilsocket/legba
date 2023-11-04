use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use ldap3::{LdapConnAsync, LdapConnSettings};

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("ldap", Box::new(LDAP::new()));
}

#[derive(Clone)]
pub(crate) struct LDAP {
    host: String,
    port: u16,
    domain: String,
    url: String,
}

impl LDAP {
    pub fn new() -> Self {
        LDAP {
            host: String::new(),
            domain: String::new(),
            url: String::new(),
            port: 389,
        }
    }
}

#[async_trait]
impl Plugin for LDAP {
    fn description(&self) -> &'static str {
        "LDAP password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 389)?;
        self.url = format!("ldap://{}:{}", &self.host, &self.port);
        self.domain = if let Some(domain) = &opts.ldap.ldap_domain {
            // example.org -> dc=example,dc=org
            format!(
                "dc={}",
                domain.split('.').collect::<Vec<&str>>().join(",dc=")
            )
        } else {
            return Err("no --ldap-domain specified".to_string());
        };

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let (conn, mut ldap) = LdapConnAsync::with_settings(
            LdapConnSettings::new()
                // .set_starttls(true)
                .set_conn_timeout(timeout),
            //.set_no_tls_verify(true),
            &self.url,
        )
        .await
        .map_err(|e| e.to_string())?;

        ldap3::drive!(conn);

        // attempts a simple bind using the passed in values of username and password
        if let Ok(res) = ldap
            .simple_bind(
                &format!("cn={},{}", &creds.username, &self.domain),
                &creds.password,
            )
            .await
        {
            return Ok(if res.success().is_ok() {
                Some(Loot::from(
                    &self.url,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                ))
            } else {
                None
            });
        }

        Ok(None)
    }
}
