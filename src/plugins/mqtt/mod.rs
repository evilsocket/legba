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
    use_v5: bool,
}

impl Mqtt {
    pub fn new() -> Self {
        Mqtt {
            client_id: "legba".to_string(),
            use_v5: false,
        }
    }

    fn is_failed_attempt(&self, err: &str) -> bool {
        if self.use_v5 {
            // MQTT v5.x
            // Check: https://docs.emqx.com/en/cloud/latest/connect_to_deployments/mqtt_client_error_codes.html#mqtt-v5-0
            err.contains("[135] CONNACK return code") || err.contains("[134] CONNACK return code")
        } else {
            // MQTT v3.x
            // Check: https://docs.emqx.com/en/cloud/latest/connect_to_deployments/mqtt_client_error_codes.html#connack-packet
            err.contains("[5] CONNACK return code") || err.contains("[4] CONNACK return code")
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

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
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
        .user_name(creds.username.to_owned())
        .password(creds.password.to_owned())
        .finalize();

        if let Err(err) = cli.connect(conn_opts).await {
            let err = err.to_string();
            if self.is_failed_attempt(&err) {
                Ok(None)
            } else {
                Err(err)
            }
        } else {
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
}
