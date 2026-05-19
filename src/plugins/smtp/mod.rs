use std::time::Duration;

use async_smtp::{SmtpClient, SmtpTransport, authentication};
use async_trait::async_trait;
use tokio::io::BufStream;

use crate::Options;
use crate::Plugin;
use crate::session::{Error, Loot};

use crate::creds::Credentials;
use crate::utils;

pub(crate) mod ntlm;
pub(crate) mod options;

super::manager::register_plugin! {
    "smtp" => SMTP::new()
}

/// Authentication mechanism resolved from --smtp-mechanism.
#[derive(Clone, Debug)]
enum Mechanism {
    /// PLAIN/LOGIN/XOAUTH2 — delegated to async-smtp.
    Sasl(authentication::Mechanism),
    /// NTLM v1 or v2 — handled by the local raw client in [`ntlm`].
    Ntlm(ntlm::Version),
}

#[derive(Clone, Debug)]
pub(crate) struct SMTP {
    mechanism: Mechanism,
    starttls: bool,
    ntlm_domain: String,
    ntlm_workstation: String,
}

impl SMTP {
    pub fn new() -> Self {
        SMTP {
            mechanism: Mechanism::Sasl(authentication::Mechanism::Plain),
            starttls: false,
            ntlm_domain: String::new(),
            ntlm_workstation: String::new(),
        }
    }

    fn loot_for(creds: &Credentials, address: &str) -> Vec<Loot> {
        vec![Loot::new(
            "smtp",
            address,
            [
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ],
        )]
    }
}

#[async_trait]
impl Plugin for SMTP {
    fn description(&self) -> &'static str {
        "SMTP password authentication."
    }

    async fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.mechanism = match opts.smtp.smtp_mechanism.to_ascii_uppercase().as_str() {
            "PLAIN" => Mechanism::Sasl(authentication::Mechanism::Plain),
            "LOGIN" => Mechanism::Sasl(authentication::Mechanism::Login),
            "XOAUTH2" => Mechanism::Sasl(authentication::Mechanism::Xoauth2),
            "NTLM" | "NTLMV2" => Mechanism::Ntlm(ntlm::Version::V2),
            "NTLMV1" => Mechanism::Ntlm(ntlm::Version::V1),
            _ => {
                return Err(format!(
                    "'{}' is not a valid SMTP authentication mechanism: expected PLAIN, LOGIN, XOAUTH2, NTLM (NTLMv2) or NTLMv1.",
                    &opts.smtp.smtp_mechanism
                ));
            }
        };
        self.starttls = opts.smtp.smtp_starttls;
        self.ntlm_domain = opts.smtp.smtp_ntlm_domain.clone();
        self.ntlm_workstation = opts.smtp.smtp_ntlm_workstation.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 25)?;

        match &self.mechanism {
            Mechanism::Sasl(mech) => {
                let host = address
                    .rsplit_once(':')
                    .map(|(h, _)| h.to_string())
                    .unwrap_or_else(|| address.clone());
                let stream =
                    crate::utils::net::async_tcp_stream(&address, &host, timeout, false).await?;
                let client = SmtpClient::new();
                let mut transport = tokio::time::timeout(
                    timeout,
                    SmtpTransport::new(client, BufStream::new(stream)),
                )
                .await
                .map_err(|e: tokio::time::error::Elapsed| e.to_string())?
                .map_err(|e| e.to_string())?;

                if self.starttls {
                    // async-smtp's starttls() sends STARTTLS, validates 220, and
                    // returns the raw stream (the BufStream we passed in).
                    let buf = transport.starttls().await.map_err(|e| e.to_string())?;
                    let plain = buf.into_inner();
                    let tls = crate::utils::net::upgrade_tcp_stream_to_ssl(plain, &host, timeout)
                        .await?;
                    // Post-STARTTLS the server does not re-greet; use
                    // .without_greeting() so SmtpTransport::new doesn't hang
                    // waiting for a 220 that won't come.
                    transport = tokio::time::timeout(
                        timeout,
                        SmtpTransport::new(
                            SmtpClient::new().without_greeting(),
                            BufStream::new(tls),
                        ),
                    )
                    .await
                    .map_err(|e: tokio::time::error::Elapsed| e.to_string())?
                    .map_err(|e| e.to_string())?;
                }

                let credentials = authentication::Credentials::new(
                    creds.username.clone(),
                    creds.password.clone(),
                );

                if transport.auth(*mech, &credentials).await.is_ok() {
                    Ok(Some(Self::loot_for(creds, &address)))
                } else {
                    Ok(None)
                }
            }
            Mechanism::Ntlm(version) => {
                let ok = ntlm::attempt(
                    &address,
                    creds,
                    &self.ntlm_domain,
                    &self.ntlm_workstation,
                    *version,
                    self.starttls,
                    timeout,
                )
                .await?;
                if ok {
                    Ok(Some(Self::loot_for(creds, &address)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
