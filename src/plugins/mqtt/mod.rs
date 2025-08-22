use rand::{Rng, distr::Alphanumeric, rng};
use std::time::Duration;

use async_trait::async_trait;
use rumqttc::{AsyncClient, MqttOptions, Transport, TlsConfiguration};

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

    // Creates an insecure TLS configuration that skips certificate verification.
    fn create_insecure_tls_config() -> TlsConfiguration {
        use rumqttc::tokio_rustls::rustls;
        
        #[derive(Debug)]
        struct DangerousAcceptor;
        
        impl rustls::client::danger::ServerCertVerifier for DangerousAcceptor {
            fn verify_server_cert(
                &self,
                _end_entity: &rustls::pki_types::CertificateDer<'_>,
                _intermediates: &[rustls::pki_types::CertificateDer<'_>],
                _server_name: &rustls::pki_types::ServerName<'_>,
                _ocsp_response: &[u8],
                _now: rustls::pki_types::UnixTime,
            ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
                Ok(rustls::client::danger::ServerCertVerified::assertion())
            }
            
            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &rustls::pki_types::CertificateDer<'_>,
                _dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }
            
            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &rustls::pki_types::CertificateDer<'_>,
                _dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }
            
            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                vec![
                    rustls::SignatureScheme::RSA_PKCS1_SHA256,
                    rustls::SignatureScheme::RSA_PKCS1_SHA384,
                    rustls::SignatureScheme::RSA_PKCS1_SHA512,
                    rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                    rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                    rustls::SignatureScheme::RSA_PSS_SHA256,
                    rustls::SignatureScheme::RSA_PSS_SHA384,
                    rustls::SignatureScheme::RSA_PSS_SHA512,
                ]
            }
        }
        
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(std::sync::Arc::new(DangerousAcceptor))
            .with_no_client_auth();
        
        TlsConfiguration::Rustls(std::sync::Arc::new(config))
    }

    async fn test_v4(
        &self,
        address: &str,
        port: u16,
        timeout: Duration,
        creds: &Credentials,
    ) -> Result<bool, Error> {
        // generate a random Client ID to avoid duplicate ID issues on connects
        let random_id: String = rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let mut options = MqttOptions::new(&random_id, address, port);

        options.set_credentials(creds.username.to_owned(), creds.password.to_owned());
        options.set_clean_session(true);
        options.set_keep_alive(timeout); 

        if self.opts.mqtt_ssl {
             // Use insecure TLS configuration that skips certificate verification
            options.set_transport(Transport::Tls(Self::create_insecure_tls_config()));
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

    async fn test_v5(
        &self,
        address: &str,
        port: u16,
        timeout: Duration,
        creds: &Credentials,
    ) -> Result<bool, Error> {
        use rumqttc::v5::{AsyncClient, MqttOptions};

        // generate a random Client ID to avoid duplicate ID issues on connects
        let random_id: String = rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let mut options = MqttOptions::new(&random_id, address, port);

        options.set_credentials(creds.username.to_owned(), creds.password.to_owned());
        options.set_clean_start(true);
        options.set_keep_alive(timeout); // 

        if self.opts.mqtt_ssl {
            // Use insecure TLS configuration that skips certificate verification
            options.set_transport(Transport::Tls(Self::create_insecure_tls_config()));
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