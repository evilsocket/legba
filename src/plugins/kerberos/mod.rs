use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ahash::HashSet;
use async_trait::async_trait;
use kerberos_asn1::{AsRep, Asn1Object, KrbError};
use kerberos_constants::error_codes;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
use crate::utils;
use transport::Protocol;

mod builder;
mod transport;

pub(crate) mod options;

super::manager::register_plugin! {
    "kerberos" => Kerberos::new()
}

#[derive(Clone)]
pub(crate) struct Kerberos {
    realm: String,
    proto: Protocol,
    linux: bool,
    invalid_users: Arc<RwLock<HashSet<String>>>,
}

impl Kerberos {
    pub fn new() -> Self {
        Kerberos {
            realm: String::new(),
            proto: Protocol::default(),
            linux: false,
            invalid_users: Arc::new(RwLock::new(HashSet::default())),
        }
    }

    fn handle_error(
        &self,
        server: &SocketAddr,
        raw: &[u8],
        creds: &Credentials,
    ) -> (bool, bool, Option<Vec<Loot>>) {
        if let Ok((_, krb_error)) = KrbError::parse(raw) {
            match krb_error.error_code {
                error_codes::KDC_ERR_PREAUTH_FAILED => {
                    // found a valid username
                    return (
                        true,
                        true,
                        Some(vec![Loot::new(
                            "kerberos",
                            &server.to_string(),
                            [("username".to_owned(), creds.username.to_owned())],
                        )
                        .set_partial()]),
                    );
                }
                error_codes::KDC_ERR_KEY_EXPIRED => {
                    // valid but expired
                    return (
                        true,
                        false,
                        Some(vec![Loot::new(
                            "kerberos",
                            &server.to_string(),
                            [
                                ("username".to_owned(), creds.username.to_owned()),
                                ("expired_password".to_owned(), creds.password.to_owned()),
                            ],
                        )
                        .set_partial()]),
                    );
                }
                error_codes::KDC_ERR_CLIENT_REVOKED => {
                    // valid but revoked
                    return (
                        true,
                        false,
                        Some(vec![Loot::new(
                            "kerberos",
                            &server.to_string(),
                            [
                                ("username".to_owned(), creds.username.to_owned()),
                                ("revoked_password".to_owned(), creds.password.to_owned()),
                            ],
                        )
                        .set_partial()]),
                    );
                }
                _ => {
                    return (true, false, None);
                }
            }
        }

        (false, false, None)
    }

    fn handle_as_rep(
        &self,
        server: &SocketAddr,
        raw: &[u8],
        creds: &Credentials,
    ) -> (bool, Option<Vec<Loot>>) {
        if AsRep::parse(raw).is_ok() {
            return (
                true,
                Some(vec![Loot::new(
                    "kerberos",
                    &server.to_string(),
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                        // ("ticket".to_owned(), format!("{:?}", &as_rep.ticket)),
                    ],
                )]),
            );
        }

        (false, None)
    }
}

#[async_trait]
impl Plugin for Kerberos {
    fn description(&self) -> &'static str {
        "Kerberos 5 (pre)authentication and users enumeration."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.realm = if let Some(realm) = &opts.kerberos.kerberos_realm {
            realm.clone()
        } else {
            return Err("no --kerberos-realm argument provided".to_owned());
        };
        self.linux = opts.kerberos.kerberos_linux;
        self.proto = opts.kerberos.kerberos_protocol.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        // make sure we don't iterate over users that have been flagged as invalid
        if self.invalid_users.read().unwrap().contains(&creds.username) {
            return Ok(None);
        }

        let address = utils::parse_target_address(&creds.target, 88)?;
        let server = address
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .next()
            .ok_or("could not convert target address to socket address".to_owned())
            .map_err(|e| e.to_string())?;

        // create an AS-REQ message to get an AS-REP response
        let req = builder::create_as_req(&self.realm, creds, self.linux);

        // create transport channel, connect and send AS-REQ
        let transport = transport::get(&self.proto, server);
        let raw_resp = transport
            .request(timeout, &req.build())
            .map_err(|e| e.to_string())?;

        // did we get an error?
        let (is_error, is_valid_user, loot) = self.handle_error(&server, &raw_resp, creds);
        if is_error {
            // if this username is not valid, just mark for skipping
            if !is_valid_user {
                self.invalid_users
                    .write()
                    .unwrap()
                    .insert(creds.username.to_owned());
            }
            return Ok(loot);
        }

        // did we get an AS-REP?
        let (is_as_rep, loot) = self.handle_as_rep(&server, &raw_resp, creds);
        if is_as_rep {
            return Ok(loot);
        }

        // this shouldn't happen
        log::error!("unexpected response to AS-REQ {:?}", raw_resp);

        Ok(None)
    }
}
