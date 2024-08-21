use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

use async_trait::async_trait;
use grabbers::grab_udp_banner;
use tokio::net::UdpSocket;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;
use crate::{creds, utils};

use crate::creds::{Credentials, Expression};

use super::plugin::PayloadStrategy;

mod grabbers;
pub(crate) mod options;

super::manager::register_plugin! {
    "port.scanner" => PortScanner::new()
}

#[derive(Clone)]
pub(crate) struct PortScanner {
    ports: Expression,
    opts: options::Options,
}

impl PortScanner {
    pub fn new() -> Self {
        PortScanner {
            ports: Expression::default(),
            opts: options::Options::default(),
        }
    }

    async fn tcp_attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Loot>, Error> {
        let (target, _) = utils::parse_target(&creds.target, 0)?;
        let address = format!("{}:{}", &target, &creds.username); // username is the port
        let start: std::time::Instant = std::time::Instant::now();

        if let Ok(stream) = crate::utils::net::async_tcp_stream(&address, timeout, false).await {
            let mut data = vec![
                ("transport".to_owned(), "tcp".to_owned()),
                ("port".to_owned(), creds.username.to_owned()),
                ("time".to_owned(), format!("{:?}", start.elapsed())),
            ];

            if !self.opts.port_scanner_no_banners {
                let banner = grabbers::grab_tcp_banner(
                    &self.opts,
                    &target,
                    creds.username.parse::<u16>().unwrap(),
                    stream,
                    std::time::Duration::from_millis(self.opts.port_scanner_banner_timeout),
                )
                .await;

                for (key, val) in banner {
                    if key == "proto" || key == "protocol" {
                        data.push(("protocol".to_owned(), val));
                    } else if key.starts_with("certificate.") {
                        data.push((key, val));
                    } else {
                        data.push((format!("banner.{}", key), val));
                    }
                }
            }

            Ok(Some(Loot::new("port.scanner", &target, data)))
        } else {
            Ok(None)
        }
    }

    fn get_socket_address(&self, target: &str, creds: &Credentials) -> Result<SocketAddr, Error> {
        let address = format!("{}:{}", target, &creds.username); // username is the port
        let addresses: Vec<SocketAddr> = address
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .collect();

        // prioritize ipv4
        for addr in &addresses {
            if addr.is_ipv4() {
                return Ok(*addr);
            }
        }

        if addresses.is_empty() {
            Err(format!("can't get socket address for {target}"))
        } else {
            Ok(addresses[0])
        }
    }

    async fn udp_attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Loot>, Error> {
        let (target, _) = utils::parse_target(&creds.target, 0)?;
        let socket = self.get_socket_address(&target, creds)?;
        let start: std::time::Instant = std::time::Instant::now();

        let local_addr = match socket {
            SocketAddr::V4(_) => "0.0.0.0:0"
                .parse::<SocketAddr>()
                .map_err(|e| e.to_string())?,
            SocketAddr::V6(_) => "[::]:0".parse::<SocketAddr>().map_err(|e| e.to_string())?,
        };

        if let Ok(Ok(udp_socket)) =
            tokio::time::timeout(timeout, UdpSocket::bind(&local_addr)).await
        {
            let mut buf = [0u8; 1024];

            tokio::time::timeout(timeout, udp_socket.connect(socket))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

            tokio::time::timeout(
                timeout,
                udp_socket.send(grabbers::dns::CHAOS_BIND_VERSION_QUERY),
            )
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

            if let Ok(Ok(size)) = tokio::time::timeout(timeout, udp_socket.recv(&mut buf)).await {
                let mut data = vec![
                    ("transport".to_owned(), "udp".to_owned()),
                    ("port".to_owned(), creds.username.to_owned()),
                    ("time".to_owned(), format!("{:?}", start.elapsed())),
                ];

                for (name, value) in grab_udp_banner(&buf[0..size]).await {
                    data.push((name, value));
                }

                return Ok(Some(Loot::new("port.scanner", &target, data)));
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl Plugin for PortScanner {
    fn description(&self) -> &'static str {
        "TCP and UDP ports scanner."
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::Single
    }

    fn override_payload(&self) -> Option<Expression> {
        if self.ports.is_default() {
            Some(creds::parse_expression(Some(
                &options::DEFAULT_PORTS.to_owned(),
            )))
        } else {
            Some(self.ports.clone())
        }
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.ports = if opts.username.is_some() {
            creds::parse_expression(opts.username.as_ref())
        } else {
            creds::parse_expression(Some(&opts.port_scanner.port_scanner_ports))
        };

        if !matches!(
            &self.ports,
            Expression::Range {
                min: _,
                max: _,
                set: _
            } | Expression::Multiple { expressions: _ }
                | Expression::Constant { value: _ }
        ) {
            return Err(format!(
                "'{:?}' is not a valid port range expression",
                &self.ports
            ));
        }

        self.opts = opts.port_scanner.clone();

        if self.opts.port_scanner_no_tcp && self.opts.port_scanner_no_udp {
            Err("both TCP and UDP port scanning are disabled".to_string())
        } else {
            Ok(())
        }
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let mut loot = vec![];

        if !self.opts.port_scanner_no_udp {
            if let Ok(Some(udp_loot)) = self.udp_attempt(creds, timeout).await {
                loot.push(udp_loot);
            }
        }

        if !self.opts.port_scanner_no_tcp {
            if let Ok(Some(tcp_loot)) = self.tcp_attempt(creds, timeout).await {
                loot.push(tcp_loot);
            }
        }

        Ok(if loot.is_empty() { None } else { Some(loot) })
    }
}
