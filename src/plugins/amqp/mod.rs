use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

const PROTOCOL_HEADER_091: &[u8] = &[b'A', b'M', b'Q', b'P', 0, 0, 9, 1];

#[ctor]
fn register() {
    crate::plugins::manager::register("amqp", Box::new(AMQP::new()));
}

#[derive(Clone)]
pub(crate) struct AMQP {
    host: String,
    port: u16,
    address: String,
}

impl AMQP {
    pub fn new() -> Self {
        AMQP {
            host: String::new(),
            port: 5672,
            address: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for AMQP {
    fn description(&self) -> &'static str {
        "AMQP password authentication (ActiveMQ, RabbitMQ, Qpid, JORAM and Solace)."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 5672)?;
        self.address = format!("{}:{}", &self.host, self.port);
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        // TODO: SSL
        let mut stream = tokio::time::timeout(timeout, TcpStream::connect(&self.address))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        // send proto header
        stream
            .write_all(PROTOCOL_HEADER_091)
            .await
            .map_err(|e| e.to_string())?;

        // read connection.start header
        let mut conn_start_header = [0_u8; 7];
        stream
            .read_exact(&mut conn_start_header)
            .await
            .map_err(|e| e.to_string())?;
        let size_raw: [u8; 4] = conn_start_header[3..].try_into().unwrap();
        let payload_size = u32::from_be_bytes(size_raw) + 1;
        // read connection.start body
        let mut conn_start_body = vec![0_u8; payload_size as usize];
        stream
            .read_exact(&mut conn_start_body)
            .await
            .map_err(|e| e.to_string())?;

        // send connection.start-ok
        let auth = [
            &[0],
            creds.username.as_bytes(),
            &[0],
            creds.password.as_bytes(),
        ]
        .concat();

        let frame_args = [
            &vec![0x00, 0x00, 0x00, 0x00],              // 0 client properties
            &vec![0x05, b'P', b'L', b'A', b'I', b'N'],  // mechanism
            (auth.len() as u32).to_be_bytes().as_ref(), // auth len + auth
            &auth,
            &vec![0x05, b'e', b'n', b'_', b'U', b'S'], // locale
        ]
        .concat();

        let frame = [
            &[0x01, 0, 0],                                          // type:method + channel: 0
            ((frame_args.len() + 4) as u32).to_be_bytes().as_ref(), // length
            &[0, 0x0a],                                             // class: connection
            &[0, 0x0b],                                             // method: start-ok
            &frame_args,
            &[0xce], // frame end
        ]
        .concat();

        stream.write_all(&frame).await.map_err(|e| e.to_string())?;

        // read response
        let mut buffer = [0_u8; 16];
        stream.read(&mut buffer).await.map_err(|e| e.to_string())?;

        if buffer[0] == 0x01 {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else {
            Ok(None)
        }
    }
}
