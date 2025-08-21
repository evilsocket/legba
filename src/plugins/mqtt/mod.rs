use std::time::Duration;
use tokio_rustls::rustls::ClientConfig;
use rand::{distr::Alphanumeric, Rng, rng};


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

trait Client {

}

#[derive(Clone)]
pub(crate) struct Mqtt {
    opts: options::Options,
}

impl Mqtt {
    pub fn new() -> Self {
        Mqtt {
            opts: options::Options::default(),
        }
    }

    fn create_client(&self) -> Result<Box<dyn Client>, Error> {
        use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Transport};

    }
}

#[async_trait]
impl Plugin for Mqtt {
    fn description(&self) -> &'static str {
        "MQTT password authentication with optional SSL/TLS support."
    }

    async fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.opts = opts.mqtt.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        // Select default port based on SSL usage
        let default_port = if self.opts.mqtt_ssl { 8883 } else { 1883 };
        let address = utils::parse_target_address(&creds.target, default_port)?;
        // generate a random  Client ID to avoid duplicate ID issues on connects
        let random_id: String = rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        Ok(None)

        /*
        // Create async MQTT client
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(uri)
            .client_id(dynamic_client_id)
            .finalize();

        let cli = mqtt::AsyncClient::new(create_opts).map_err(|e| e.to_string())?;

        // Build connection options (inside scope to drop builder before .await)
        let conn_opts = {
            let mut conn_opts_builder = if self.use_v5 {
                mqtt::ConnectOptionsBuilder::new_v5()
            } else {
                mqtt::ConnectOptionsBuilder::new() // MQTT v3.x
            };

            conn_opts_builder
                .connect_timeout(timeout)
                .user_name(&creds.username)
                .password(&creds.password)
                .clean_session(true); // Ensure broker clears session on disconnect

            // Configure SSL/TLS options if SSL is enabled
            if self.use_ssl {
                // Equivalent to MQTTX settings: SSL/TLS ON + SSL Secure OFF
                // This will skip server certificate validation for testing/self-signed certs
                let ssl_opts = mqtt::SslOptionsBuilder::new()
                    .enable_server_cert_auth(false)
                    .finalize();

                conn_opts_builder.ssl_options(ssl_opts);
            }

            conn_opts_builder.finalize()
        };

        // Attempt connection to the broker
        if let Err(err) = cli.connect(conn_opts).await {
            match err {
                // n=-1 means timeout or general connection failure (retryable)
                // See: https://github.com/eclipse/paho.mqtt.c/blob/master/src/MQTTClient.h
                paho_mqtt::Error::Failure => Err(err.to_string()),
                // Other errors indicate authentication or protocol issues (non-retryable)
                _ => Ok(None),
            }
        } else {
            // Successfully connected - disconnect immediately
            let _ = cli.disconnect(None).await;

            // Save the valid credentials in Loot
            Ok(Some(vec![Loot::new(
                "mqtt",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
        }
        */
    }
}