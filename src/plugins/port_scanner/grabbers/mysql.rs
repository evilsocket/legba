use std::time::Duration;

use lazy_regex::{bytes_lazy_regex, Lazy};
use tokio::io::AsyncReadExt;

use super::Banner;
use crate::utils::net::StreamLike;

static BANNER_PARSER: Lazy<regex::bytes::Regex> =
    bytes_lazy_regex!(r"(?-u).{4}\x0a([^\x00]+)\x00.+");

pub(crate) fn is_mysql_port(port: u16) -> bool {
    port == 3306
}

pub(crate) async fn tcp_grabber(
    address: &str,
    port: u16,
    mut stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Banner {
    log::debug!("grabbing mysql banner from {}:{} ...", address, port);

    let mut banner = Banner::default();

    banner.insert("protocol".to_owned(), "mysql".to_owned());

    let mut buf = [0u8; 80];
    if let Ok(Ok(read)) = tokio::time::timeout(timeout, stream.read(&mut buf)).await {
        if read > 0 {
            for cap in BANNER_PARSER.captures_iter(&buf[0..read]) {
                banner.insert(
                    "mysql.version".to_owned(),
                    String::from_utf8_lossy(cap.get(1).unwrap().as_bytes()).to_string(),
                );
            }
        }
    }

    banner
}
