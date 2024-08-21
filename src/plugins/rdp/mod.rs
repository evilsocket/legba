use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use async_trait::async_trait;
use rdp::core::client::Connector;
use rdp::core::gcc::KeyboardLayout;

use crate::session::{Error, Loot};
use crate::Plugin;
use crate::{utils, Options};

use crate::creds::Credentials;

pub(crate) mod options;

super::manager::register_plugin! {
    "rdp" => RDP::new()
}

#[derive(Clone)]
pub(crate) struct RDP {
    options: options::Options,
}

impl RDP {
    pub fn new() -> Self {
        RDP {
            options: options::Options::default(),
        }
    }
}

#[async_trait]
impl Plugin for RDP {
    fn description(&self) -> &'static str {
        "Microsoft Remote Desktop password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.options = opts.rdp.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let (host, port) = utils::parse_target(&creds.target, 3389)?;
        let address = format!("{}:{}", &host, port)
            .parse::<SocketAddr>()
            .map_err(|e| e.to_string())?;

        let stream = TcpStream::connect_timeout(&address, timeout).map_err(|e| e.to_string())?;

        let mut rdp_connector = Connector::new()
            .screen(800, 600)
            .credentials(
                self.options.rdp_domain.to_owned(),
                creds.username.to_owned(),
                creds.password.to_owned(),
            )
            .layout(KeyboardLayout::US)
            .set_restricted_admin_mode(self.options.rdp_admin_mode)
            .auto_logon(self.options.rdp_auto_logon)
            .check_certificate(false);

        if self.options.rdp_ntlm {
            rdp_connector = rdp_connector.set_password_hash(
                hex::decode(&creds.password)
                    .map_err(|e| format!("cannot parse the input hash [{}]", e))?,
            );
        }

        if rdp_connector.connect(stream).is_ok() {
            Ok(Some(vec![Loot::new(
                "rdp",
                &address.to_string(),
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
