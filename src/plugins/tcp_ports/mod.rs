use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;
use crate::{creds, utils};

use crate::creds::{Credentials, Expression};

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("tcp.ports", Box::new(TcpPortScanner::new()));
}

#[derive(Clone)]
pub(crate) struct TcpPortScanner {
    ports: Expression,
}

impl TcpPortScanner {
    pub fn new() -> Self {
        TcpPortScanner {
            ports: Expression::default(),
        }
    }
}

#[async_trait]
impl Plugin for TcpPortScanner {
    fn description(&self) -> &'static str {
        "TCP connect ports scanner."
    }

    fn single_credential(&self) -> bool {
        true
    }

    fn override_payload(&self) -> Option<Expression> {
        Some(self.ports.clone())
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.ports = creds::parse_expression(Some(&format!("[{}]", &opts.tcp_ports.tcp_ports)));
        if !matches!(
            &self.ports,
            Expression::Range {
                min: _,
                max: _,
                set: _
            }
        ) {
            return Err(format!(
                "'{}' is not a valid port range expression",
                &opts.tcp_ports.tcp_ports
            ));
        }

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let (target, _) = utils::parse_target(&creds.target, 0)?;
        let address = format!("{}:{}", &target, &creds.username); // username is the port
        let start: std::time::Instant = std::time::Instant::now();

        return if crate::utils::net::async_tcp_stream(&address, timeout, false)
            .await
            .is_ok()
        {
            Ok(Some(Loot::new(
                "tcp.ports",
                &target,
                [
                    ("proto".to_owned(), "tcp".to_owned()),
                    ("port".to_owned(), creds.username.to_owned()),
                    ("time".to_owned(), format!("{:?}", start.elapsed())),
                ],
            )))
        } else {
            Ok(None)
        };
    }
}
