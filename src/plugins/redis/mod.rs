use std::time::Duration;
use std::sync::Mutex;
use std::collections::HashMap;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use lazy_static::lazy_static;

use crate::Plugin;
use crate::session::{Error, Loot};
use crate::{Options, utils};
use crate::creds::Credentials;

pub(crate) mod options;

// Global cache for storing authentication type per target
lazy_static! {
    static ref AUTH_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

super::manager::register_plugin! {
    "redis" => Redis::new()
}

#[derive(Clone)]
pub(crate) struct Redis {
    ssl: bool,
}

impl Redis {
    pub fn new() -> Self {
        Redis { ssl: false }
    }
}

#[async_trait]
impl Plugin for Redis {
    fn description(&self) -> &'static str {
        "Redis legacy and ACL password authentication."
    }

    async fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.ssl = opts.redis.redis_ssl;
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 6379)?;
        
        // Check if authentication type is cached
        let auth_type = {
            let cache = AUTH_CACHE.lock().map_err(|e| e.to_string())?;
            cache.get(&address).cloned()
        };

        if let Some(auth_type) = auth_type {
            // Use cached authentication type
            match auth_type.as_str() {
                "none" => {
                    // Redis has no authentication, skip password attempts
                    return Ok(None);
                }
                "password_only" => {
                    if creds.password.is_empty() {
                        return Ok(None);
                    }
                    let mut stream =
                        crate::utils::net::async_tcp_stream(&address, "", timeout, self.ssl).await?;
                    stream
                        .write_all(format!("AUTH {}\r\n", &creds.password).as_bytes())
                        .await
                        .map_err(|e| e.to_string())?;
                    let mut buffer = [0_u8; 1024];
                    let n = stream.read(&mut buffer).await.map_err(|e| e.to_string())?;
                    let response = std::str::from_utf8(&buffer[..n]).unwrap_or("");
                    if response.starts_with("+OK") {
                        return Ok(Some(vec![Loot::new(
                            "redis",
                            &address,
                            [
                                ("auth_type".to_owned(), "password_only".to_owned()),
                                ("password".to_owned(), creds.password.to_owned()),
                            ],
                        )]));
                    }
                }
                "acl" => {
                    if creds.password.is_empty() {
                        return Ok(None);
                    }
                    let username = if creds.username.is_empty() { "default" } else { &creds.username };
                    let mut stream =
                        crate::utils::net::async_tcp_stream(&address, "", timeout, self.ssl).await?;
                    stream
                        .write_all(format!("AUTH {} {}\r\n", username, &creds.password).as_bytes())
                        .await
                        .map_err(|e| e.to_string())?;
                    let mut buffer = [0_u8; 1024];
                    let n = stream.read(&mut buffer).await.map_err(|e| e.to_string())?;
                    let response = std::str::from_utf8(&buffer[..n]).unwrap_or("");
                    if response.starts_with("+OK") {
                        return Ok(Some(vec![Loot::new(
                            "redis",
                            &address,
                            [
                                ("auth_type".to_owned(), "acl".to_owned()),
                                ("username".to_owned(), username.to_owned()),
                                ("password".to_owned(), creds.password.to_owned()),
                            ],
                        )]));
                    }
                }
                _ => {}
            }
            return Ok(None);
        }

        // No cached authentication type, perform full check
        let mut stream =
            crate::utils::net::async_tcp_stream(&address, "", timeout, self.ssl).await?;

        // Try PING to check if auth is required
        stream
            .write_all(b"PING\r\n")
            .await
            .map_err(|e| e.to_string())?;

        let mut buffer = [0_u8; 1024];
        let n = stream
            .read(&mut buffer)
            .await
            .map_err(|e| e.to_string())?;

        let response = std::str::from_utf8(&buffer[..n]).unwrap_or("");

        // If we get +PONG, no auth is required
        if response.starts_with("+PONG") {
            let mut cache = AUTH_CACHE.lock().map_err(|e| e.to_string())?;
            cache.insert(address.clone(), "none".to_string());
            return Ok(Some(vec![Loot::new(
                "redis",
                &address,
                [
                    ("auth_type".to_owned(), "none".to_owned()),
                    ("info".to_owned(), "No authentication required".to_owned()),
                ],
            )]));
        }

        // If we get -NOAUTH or -ERR, authentication is required
        if response.contains("NOAUTH") || response.contains("ERR") {
            if creds.password.is_empty() {
                return Ok(None);
            }

            // Try password-only auth first (legacy Redis < 6.0)
            stream
                .write_all(format!("AUTH {}\r\n", &creds.password).as_bytes())
                .await
                .map_err(|e| e.to_string())?;

            let n = stream
                .read(&mut buffer)
                .await
                .map_err(|e| e.to_string())?;

            let response = std::str::from_utf8(&buffer[..n]).unwrap_or("");

            if response.starts_with("+OK") {
                let mut cache = AUTH_CACHE.lock().map_err(|e| e.to_string())?;
                cache.insert(address.clone(), "password_only".to_string());
                return Ok(Some(vec![Loot::new(
                    "redis",
                    &address,
                    [
                        ("auth_type".to_owned(), "password_only".to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                )]));
            }

            // If password-only failed with wrong number of arguments, try ACL auth
            if response.contains("wrong number of arguments") || 
               (response.contains("ERR") && !creds.username.is_empty()) {
                
                let username = if creds.username.is_empty() { "default" } else { &creds.username };

                // Create a new connection for ACL auth
                let mut stream =
                    crate::utils::net::async_tcp_stream(&address, "", timeout, self.ssl).await?;

                stream
                    .write_all(format!("AUTH {} {}\r\n", username, &creds.password).as_bytes())
                    .await
                    .map_err(|e| e.to_string())?;

                let n = stream
                    .read(&mut buffer)
                    .await
                    .map_err(|e| e.to_string())?;

                let response = std::str::from_utf8(&buffer[..n]).unwrap_or("");

                if response.starts_with("+OK") {
                    let mut cache = AUTH_CACHE.lock().map_err(|e| e.to_string())?;
                    cache.insert(address.clone(), "acl".to_string());
                    return Ok(Some(vec![Loot::new(
                        "redis",
                        &address,
                        [
                            ("auth_type".to_owned(), "acl".to_owned()),
                            ("username".to_owned(), username.to_owned()),
                            ("password".to_owned(), creds.password.to_owned()),
                        ],
                    )]));
                }
            }
        }

        Ok(None)
    }
}