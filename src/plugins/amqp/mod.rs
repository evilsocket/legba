use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

pub(crate) mod options;

const PROTOCOL_HEADER_091: &[u8] = &[b'A', b'M', b'Q', b'P', 0, 0, 9, 1];

super::manager::register_plugin! {
    "amqp" => AMQP::new()
}

#[derive(Clone)]
pub(crate) struct AMQP {
    ssl: bool,
}

impl AMQP {
    pub fn new() -> Self {
        AMQP { ssl: false }
    }
}

#[async_trait]
impl Plugin for AMQP {
    fn description(&self) -> &'static str {
        "AMQP password authentication (ActiveMQ, RabbitMQ, Qpid, JORAM and Solace)."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.ssl = opts.amqp.amqp_ssl;
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 5672)?;
        let mut stream = crate::utils::net::async_tcp_stream(&address, timeout, self.ssl).await?;

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
            &[0x00, 0x00, 0x00, 0x00][..],              // 0 client properties
            &[0x05, b'P', b'L', b'A', b'I', b'N'],      // mechanism
            (auth.len() as u32).to_be_bytes().as_ref(), // auth len + auth
            &auth,
            &[0x05, b'e', b'n', b'_', b'U', b'S'], // locale
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
            Ok(Some(vec![Loot::new(
                "amqp",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
        } else {
            Ok(None)
        }
    }
}
