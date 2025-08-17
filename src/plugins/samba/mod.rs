use std::time::Duration;

use async_trait::async_trait;

use crate::Plugin;
use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::{Options, utils};

super::manager::register_plugin! {
    "smb" => SMB::new()
}

#[derive(Clone)]
pub(crate) struct SMB {}

impl SMB {
    pub fn new() -> Self {
        SMB {}
    }
}

#[async_trait]
impl Plugin for SMB {
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

        let mut conn = smb::Connection::build(&address, config.connection.clone())
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
            // correct user, wrong pass: Some(UnexpectedMessageStatus(3221225581))
            Err(smb::Error::UnexpectedMessageStatus(_)) => Ok(Some(vec![
                Loot::new(
                    "smb",
                    &address,
                    [("username".to_owned(), creds.username.to_owned())],
                )
                .set_partial(),
            ])),
            // wrong user: Some(InvalidMessage("Message not signed or encrypted, but signing is required for the session!"))
            Err(_) => Ok(None),
        };
    }
}
