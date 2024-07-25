use std::time::Duration;

use crate::utils::net::StreamLike;
use ahash::HashMap;

use super::options;

pub(crate) mod dns;

mod http;
mod line;
mod mysql;

pub(crate) type Banner = HashMap<String, String>;

pub(crate) async fn grab_tcp_banner(
    opts: &options::Options,
    address: &str,
    port: u16,
    stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Banner {
    if mysql::is_mysql_port(port) {
        mysql::tcp_grabber(address, port, stream, timeout).await
    } else if dns::is_dns_port(port) {
        dns::tcp_grabber(address, port, stream, timeout).await
    } else if let (true, with_ssl) = http::is_http_port(opts, port) {
        http::http_grabber(opts, address, port, stream, with_ssl, timeout).await
    } else {
        // default to an attempt at line grabbing
        line::line_grabber(address, port, stream, timeout).await
    }
}

pub(crate) async fn grab_udp_banner(response: &[u8]) -> Banner {
    dns::parse_maybe_chaos_response(response).await
}
