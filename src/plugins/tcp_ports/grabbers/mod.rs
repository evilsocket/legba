use std::time::Duration;

use crate::utils::net::StreamLike;
use ahash::HashMap;

use super::options;

mod http;
mod line;

pub(crate) type Banner = HashMap<String, String>;

pub(crate) async fn grab_banner(
    opts: &options::Options,
    address: &str,
    port: u16,
    stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Banner {
    let (is_http, with_ssl) = http::is_http_port(opts, port);
    if is_http {
        return http::http_grabber(opts, address, port, stream, with_ssl, timeout).await;
    }

    // default to an attempt at line grabbing
    line::line_grabber(address, port, stream, timeout).await
}
