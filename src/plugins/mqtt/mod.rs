use paho_mqtt as mqtt;
use std::time::Duration;

use async_trait::async_trait;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

pub(crate) mod options;

super::manager::register_plugin! {
    "mqtt" => Mqtt::new()
}

#[derive(Clone)]
pub(crate) struct Mqtt {
    client_id: String,
    use_v5: bool,
}

impl Mqtt {
    pub fn new() -> Self {
        Mqtt {
            client_id: "legba".to_string(),
            use_v5: false,
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
        self.use_v5 = opts.mqtt.mqtt_v5;
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 1883)?;
        let uri = format!("mqtt://{}", address);

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(uri)
            .client_id(self.client_id.to_owned())
            .finalize();

        let cli = mqtt::AsyncClient::new(create_opts).map_err(|e| e.to_string())?;

        let conn_opts = if self.use_v5 {
            mqtt::ConnectOptionsBuilder::new_v5()
        } else {
            mqtt::ConnectOptionsBuilder::new() // v3.x
        }
        .connect_timeout(timeout)
        .user_name(&creds.username)
        .password(&creds.password)
        .finalize();

        if let Err(err) = cli.connect(conn_opts).await {
            match err {
                // Timeouts and failed connections are reported with n=-1, in which case we return the error
                // as we want to retry --retry times.
                // See https://github.com/eclipse-paho/paho.mqtt.c/blob/2150ba29d9df24ad1733c460eb099f292af84ee5/src/MQTTClient.h#L142
                paho_mqtt::Error::Failure => Err(err.to_string()),
                // Failed logings and other protocol errors are reported with other integer codes, in which
                // case we return Ok(None) to move to the next set of credentials.
                _ => Ok(None),
            }
        } else {
            Ok(Some(vec![Loot::new(
                "mqtt",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
        }
    }
}
