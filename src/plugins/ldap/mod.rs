use std::time::Duration;

use async_trait::async_trait;
use ldap3::{LdapConnAsync, LdapConnSettings};

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;

pub(crate) mod options;

super::manager::register_plugin! {
    "ldap" => LDAP::new()
}

#[derive(Clone)]
pub(crate) struct LDAP {
    domain: String,
}

impl LDAP {
    pub fn new() -> Self {
        LDAP {
            domain: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for LDAP {
    fn description(&self) -> &'static str {
        "LDAP password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
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

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 389)?;
        let url = format!("ldap://{}", address);

        let (conn, mut ldap) = LdapConnAsync::with_settings(
            LdapConnSettings::new()
                // .set_starttls(true)
                .set_conn_timeout(timeout),
            //.set_no_tls_verify(true),
            &url,
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
                Some(vec![Loot::new(
                    "ldap",
                    &address,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )])
            } else {
                None
            });
        }

        Ok(None)
    }
}
