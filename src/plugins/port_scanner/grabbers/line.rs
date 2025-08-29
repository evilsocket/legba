use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::Banner;
use crate::{
    plugins::port_scanner::{grabbers::http::parse_http_raw_response, options},
    utils::net::StreamLike,
};
use std::time::{Duration, Instant};

async fn read_response_from(mut stream: Box<dyn StreamLike>, timeout: Duration) -> String {
    let mut response = String::new();
    let mut buf: [u8; 1] = [0];
    let max = 1024 * 4;
    let started = Instant::now();

    for _ in 0..max {
        if let Ok(read) = tokio::time::timeout(timeout, stream.read_exact(&mut buf)).await {
            if read.is_ok() {
                response.push(buf[0] as char);
            } else {
                log::debug!("{:?}", read);
                break;
            }
        }

        if started.elapsed() > timeout {
            log::debug!("timeout={:?} after {} bytes", timeout, response.len());
            break;
        }
    }

    response
}

pub(crate) async fn line_grabber(
    opts: &options::Options,
    address: &str,
    port: u16,
    mut stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Banner {
    log::debug!("grabbing line banner from {}:{} ...", address, port);

    let mut banner = Banner::default();

    // send something
    let _ = stream
        .write_all(format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", address).as_bytes())
        .await;

    let response = read_response_from(stream, timeout).await;
    if !response.is_empty() {
        // if we have an http response ...
        if response.contains("HTTP/") {
            banner.insert("protocol".to_owned(), "http".to_owned());
            parse_http_raw_response(opts, &response, &mut banner).await;
        } else {
            banner.insert(
                "data".to_owned(),
                response
                    .trim()
                    .replace("\r\n", "<crlf>")
                    .replace("\n", "<lf>"),
            );
        }
    }

    banner
}
