use rand::{Rng, distr::Alphanumeric, rng};
use std::time::Duration;

use async_trait::async_trait;

use crate::Options;
use crate::Plugin;
use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;

pub(crate) mod options;

super::manager::register_plugin! {
    "mqtt" => Mqtt::new()
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

    async fn test_v4(
        &self,
        address: &str,
        port: u16,
        timeout: Duration,
        creds: &Credentials,
    ) -> Result<bool, Error> {
        use rumqttc::{AsyncClient, MqttOptions, NetworkOptions, Transport};

        // generate a random  Client ID to avoid duplicate ID issues on connects
        let random_id: String = rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let mut options = MqttOptions::new(&random_id, address, port);

        options.set_credentials(creds.username.to_owned(), creds.password.to_owned());
        options.set_clean_session(true);

        if self.opts.mqtt_ssl {
            options.set_transport(Transport::tls_with_default_config());
        } else {
            options.set_transport(Transport::Tcp);
        }

        let (_, mut eventloop) = AsyncClient::new(options, 5);

        let mut network_options = NetworkOptions::new();

        network_options.set_connection_timeout(timeout.as_secs());

        eventloop.set_network_options(network_options);

        if eventloop.poll().await.is_ok() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn test_v5(
        &self,
        address: &str,
        port: u16,
        timeout: Duration,
        creds: &Credentials,
    ) -> Result<bool, Error> {
        use rumqttc::{
            Transport,
            v5::{AsyncClient, MqttOptions},
        };

        // generate a random  Client ID to avoid duplicate ID issues on connects
        let random_id: String = rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let mut options = MqttOptions::new(&random_id, address, port);

        options.set_credentials(creds.username.to_owned(), creds.password.to_owned());
        options.set_clean_start(true);
        options.set_connection_timeout(timeout.as_secs());

        if self.opts.mqtt_ssl {
            options.set_transport(Transport::tls_with_default_config());
        } else {
            options.set_transport(Transport::Tcp);
        }

        let (_, mut eventloop) = AsyncClient::new(options, 5);

        if eventloop.poll().await.is_ok() {
            Ok(true)
        } else {
            Ok(false)
        }
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
        let (address, port) = utils::parse_target(&creds.target, default_port)?;

        if self.opts.mqtt_v5 {
            if self
                .test_v5(&address, port, timeout, creds)
                .await
                .map_err(|e| e.to_string())?
            {
                return Ok(Some(vec![Loot::new(
                    "mqtt",
                    &address,
                    [
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )]));
            }
        } else if self
            .test_v4(&address, port, timeout, creds)
            .await
            .map_err(|e| e.to_string())?
        {
            return Ok(Some(vec![Loot::new(
                "mqtt",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]));
        }

        return Ok(None);
    }
}
