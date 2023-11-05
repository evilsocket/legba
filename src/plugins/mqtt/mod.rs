use paho_mqtt as mqtt;
use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;
pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("mqtt", Box::new(Mqtt::new()));
}

#[derive(Clone)]
pub(crate) struct Mqtt {
    client_id: String,
}

impl Mqtt {
    pub fn new() -> Self {
        Mqtt {
            client_id: "legba".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for Mqtt {
    fn description(&self) -> &'static str {
        "MQTT password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.client_id = opts.mqtt.mqtt_client_id.clone();
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let address = utils::parse_target_address(&creds.target, 1883)?;
        let uri = format!("mqtt://{}", address);

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(uri)
            .client_id(self.client_id.to_owned())
            .finalize();

        let cli = mqtt::AsyncClient::new(create_opts).map_err(|e| e.to_string())?;

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .connect_timeout(timeout)
            .user_name(creds.username.to_owned())
            .password(creds.password.to_owned())
            .finalize();

        cli.connect(conn_opts).await.map_err(|e| e.to_string())?;

        Ok(Some(Loot::new(
            "mqtt",
            &address,
            [
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ],
        )))
    }
}
