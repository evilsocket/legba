use tokio::io::AsyncReadExt;

use super::Banner;
use crate::utils::net::StreamLike;
use std::time::Duration;

async fn read_line_from(mut stream: Box<dyn StreamLike>) -> String {
    let mut line = String::new();
    let mut buf: [u8; 1] = [0];
    let max = 1024;

    for _ in 0..max {
        let read = stream.read_exact(&mut buf).await;
        if read.is_ok() {
            let c = buf[0] as char;
            if c == '\n' {
                break;
            }
            line.push(c);
        } else {
            log::debug!("{:?}", read);
            break;
        }
    }

    line
}

pub(crate) async fn line_grabber(
    address: &str,
    port: u16,
    stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Banner {
    log::debug!("grabbing line banner from {}:{} ...", address, port);

    let mut banner = Banner::default();

    let timeout = std::time::Duration::from_millis((timeout.as_millis() / 2) as u64);
    if let Ok(line) = tokio::time::timeout(timeout, read_line_from(stream)).await {
        if !line.is_empty() {
            banner.insert("line".to_owned(), line);
        }
    }

    return banner;
}
