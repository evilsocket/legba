use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::creds::{Credentials, Expression};
use crate::plugins::plugin::PayloadStrategy;
use crate::session::{Error, Loot};
use crate::{Options, utils};
use crate::{Plugin, creds};

super::manager::register_plugin! {
    "smb" => SMBAuth::new(),
    "smb.shares" => SMBShares::new()
}

#[derive(Clone)]
pub(crate) struct SMBAuth {}

impl SMBAuth {
    pub fn new() -> Self {
        SMBAuth {}
    }
}

#[async_trait]
impl Plugin for SMBAuth {
    fn description(&self) -> &'static str {
        "Samba password authentication."
    }

    async fn setup(&mut self, _: &Options) -> Result<(), Error> {
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let (address, port) = utils::parse_target(&creds.target, 445)?;

        let mut config = smb::ClientConfig::default();

        config.connection.port = Some(port);
        config.connection.timeout = Some(timeout);

        let conn = smb::Connection::build(&address, config.connection.clone())
            .map_err(|e| e.to_string())?;

        conn.connect().await.map_err(|e| e.to_string())?;

        return match conn
            .authenticate(&creds.username, creds.password.clone())
            .await
        {
            Ok(_) => Ok(Some(vec![Loot::new(
                "smb",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )])),
            Err(_) => Ok(None),
        };
    }
}

const DEFAULT_SHARES: &str = "A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z,IPC$,print,print$,localrepo,share,files,file_transfer,md0,ADMIN,BACKUP,DATA,DESKTOP,DOCS,FILES,GROUPS,HD,HOME,INFO,IPC,MEDIA,MY DOCUMENTS,NETLOGON,PICTURES,PORN,PR0N,PRINT,PROGRAMS,PRON,PUBLIC,SHARE,SHARED,SOFTWARE,STMP,TEMP,TEST,TMP,USERS,WEB DOCUMENTS,WEBSERVER,WWW,XSERVE";

lazy_static! {
    static ref SESSIONS: Mutex<DashMap<String, Arc<smb::Session>>> = Mutex::new(DashMap::new());
}

#[derive(Clone)]
pub(crate) struct SMBShares {}

impl SMBShares {
    pub fn new() -> Self {
        SMBShares {}
    }

    async fn session_for(
        &self,
        address: &str,
        port: u16,
        timeout: Duration,
    ) -> Result<Arc<smb::Session>, Error> {
        let sessions = SESSIONS.lock().await;

        if let Some(session) = sessions.get(address) {
            log::debug!("reusing session for {}", address);
            return Ok(session.clone());
        }

        let mut config = smb::ClientConfig::default();

        config.connection.port = Some(port);
        config.connection.timeout = Some(timeout);
        config.connection.allow_unsigned_guest_access = true;

        let client = smb::Client::new(config);

        if let Ok(connection) = client.connect(address).await {
            if let Ok(session) = connection.authenticate("/GUEST", "".to_string()).await {
                log::debug!("created session for {}", address);

                let session = Arc::new(session);

                sessions.insert(address.to_owned(), session.clone());

                Ok(session)
            } else {
                Err("target does not support anonymous access".to_owned())
            }
        } else {
            Err("could not connect to target".to_owned())
        }
    }
}

#[async_trait]
impl Plugin for SMBShares {
    fn description(&self) -> &'static str {
        "Samba shares enumeration."
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::Single
    }

    async fn setup(&mut self, _: &Options) -> Result<(), Error> {
        Ok(())
    }

    fn override_payload(&self) -> Option<Expression> {
        Some(creds::parse_expression(Some(&DEFAULT_SHARES.to_owned())))
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let (address, port) = utils::parse_target(&creds.target, 445)?;
        if let Ok(session) = self.session_for(&address, port, timeout).await {
            let path = format!("\\\\{address}\\{}", creds.username);
            match session.tree_connect(&path).await {
                Ok(_) => {
                    return Ok(Some(vec![Loot::new(
                        "smb.shares",
                        &address,
                        [
                            ("share".to_owned(), creds.username.to_owned()),
                            ("guest_access".to_owned(), "true".to_owned()),
                        ],
                    )]));
                }
                // if the share exists, 0xc0000022 (ACCESS_DENIED) is returned :D
                Err(smb::Error::ReceivedErrorMessage(0xc0000022, _)) => {
                    return Ok(Some(vec![Loot::new(
                        "smb.shares",
                        &address,
                        [
                            ("share".to_owned(), creds.username.to_owned()),
                            ("guest_access".to_owned(), "false".to_owned()),
                        ],
                    )]));
                }
                Err(_) => {}
            }
        }

        Ok(None)
    }
}
