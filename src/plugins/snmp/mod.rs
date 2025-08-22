use std::time::Duration;

use async_trait::async_trait;
use snmp2::AsyncSession;

use crate::Options;
use crate::Plugin;
use crate::session::{Error, Loot};
use crate::utils;

use crate::creds::Credentials;

use crate::plugins::plugin::PayloadStrategy;

pub(crate) mod oids;
pub(crate) mod options;
pub(crate) mod reader;

// v1 and v2 have no authentication, so only the community names are enumerated
// v3 has different types of authentication
crate::plugins::manager::register_plugin! {
    "snmp1" => SNMPv1::new(),
    "snmp2" => SNMPv2::new(),
    "snmp3" => SNMPv3::new()
}

#[derive(Clone)]
pub(crate) struct SNMPv1 {
    options: options::Options,
}

impl SNMPv1 {
    pub fn new() -> Self {
        SNMPv1 {
            options: options::Options::default(),
        }
    }
}

#[async_trait]
impl Plugin for SNMPv1 {
    fn description(&self) -> &'static str {
        "SNMPv1 community and OID enumeration."
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::Single
    }

    async fn setup(&mut self, options: &Options) -> Result<(), Error> {
        self.options = options.snmp.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 161)?;
        if let Ok(Ok(mut sess)) = tokio::time::timeout(
            timeout,
            AsyncSession::new_v1(&address, creds.username.as_bytes(), 0),
        )
        .await
        {
            return reader::read_from_session(
                &self.options,
                &mut sess,
                address,
                creds,
                timeout,
                None,
            )
            .await;
        }

        Ok(None)
    }
}

#[derive(Clone)]
pub(crate) struct SNMPv2 {
    options: options::Options,
}

impl SNMPv2 {
    pub fn new() -> Self {
        SNMPv2 {
            options: options::Options::default(),
        }
    }
}

#[async_trait]
impl Plugin for SNMPv2 {
    fn description(&self) -> &'static str {
        "SNMPv2 community and OID enumeration."
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::Single
    }

    async fn setup(&mut self, options: &Options) -> Result<(), Error> {
        self.options = options.snmp.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 161)?;
        if let Ok(Ok(mut sess)) = tokio::time::timeout(
            timeout,
            AsyncSession::new_v1(&address, creds.username.as_bytes(), 0),
        )
        .await
        {
            return reader::read_from_session(
                &self.options,
                &mut sess,
                address,
                creds,
                timeout,
                None,
            )
            .await;
        }

        Ok(None)
    }
}

#[derive(Clone)]
pub(crate) struct SNMPv3 {
    options: options::Options,
}

impl SNMPv3 {
    pub fn new() -> Self {
        SNMPv3 {
            options: options::Options::default(),
        }
    }
}

#[async_trait]
impl Plugin for SNMPv3 {
    fn description(&self) -> &'static str {
        "SNMPv3 username and password authentication."
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::UsernamePassword
    }

    async fn setup(&mut self, options: &Options) -> Result<(), Error> {
        self.options = options.snmp.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 161)?;
        let username = creds.username.as_bytes();
        let password = creds.password.as_bytes();

        // attempt all supported protocols
        for proto in [
            snmp2::v3::AuthProtocol::Md5,
            snmp2::v3::AuthProtocol::Sha1,
            snmp2::v3::AuthProtocol::Sha224,
            snmp2::v3::AuthProtocol::Sha256,
            snmp2::v3::AuthProtocol::Sha384,
            snmp2::v3::AuthProtocol::Sha512,
        ] {
            let security = snmp2::v3::Security::new(username, password).with_auth_protocol(proto);

            if let Ok(Ok(mut sess)) =
                tokio::time::timeout(timeout, AsyncSession::new_v3(&address, 0, security)).await
            {
                // In case if engine_id is not provided in security parameters, it is necessary
                // to call init() method to send a blank unauthenticated request to the target
                // to get the engine_id.
                if tokio::time::timeout(timeout, sess.init()).await.is_ok() {
                    return reader::read_from_session(
                        &self.options,
                        &mut sess,
                        address,
                        creds,
                        timeout,
                        Some(proto),
                    )
                    .await;
                }
            }
        }

        Ok(None)
    }
}
