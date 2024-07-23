use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::Banner;
use crate::utils::net::StreamLike;

pub(crate) const CHAOS_BIND_VERSION_QUERY: &[u8] = &[
    0xa3, 0xe0, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x07, 0x76, 0x65, 0x72,
    0x73, 0x69, 0x6f, 0x6e, 0x04, 0x62, 0x69, 0x6e, 0x64, 0x00, 0x00, 0x10, 0x00, 0x03, 0x00, 0x00,
    0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub(crate) fn is_dns_port(port: u16) -> bool {
    port == 53 || port == 5353
}

pub(crate) async fn tcp_grabber(
    _address: &str,
    _port: u16,
    mut stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Banner {
    // send request
    let _ = stream.write_all(CHAOS_BIND_VERSION_QUERY).await;
    // read response
    let mut buf = [0u8; 1024];
    if let Ok(Ok(read)) = tokio::time::timeout(timeout, stream.read(&mut buf)).await {
        parse_maybe_chaos_response(&buf[0..read]).await
    } else {
        Banner::default()
    }
}

pub(crate) async fn parse_maybe_chaos_response(response: &[u8]) -> Banner {
    let mut data = Banner::default();

    // try to parse as DNS Chaos response
    if let Ok(chaos_resp) = trust_dns_resolver::proto::op::Message::from_vec(response) {
        data.insert("protocol".to_owned(), "dns".to_owned());

        let mut found = false;
        for answer in chaos_resp.answers() {
            if answer.name().to_string() == "version.bind." && answer.data().is_some() {
                data.insert(
                    "dns.bind.version".to_owned(),
                    answer.data().unwrap().to_string(),
                );
                found = true;
                break;
            }
        }

        if !found && chaos_resp.answer_count() > 0 {
            data.insert(
                "dns.chaos.response".to_owned(),
                format!("{:?}", &chaos_resp),
            );
        }
    } else {
        data.insert("banner".to_owned(), format!("{:?}", response));
    }

    data
}
