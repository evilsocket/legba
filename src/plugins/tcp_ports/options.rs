use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "1-65535")]
    /// Range or comma separated values of integer port numbers to scan.
    pub tcp_ports: String,
    #[clap(long, default_value_t = false)]
    /// Do not attempt banner grabbing.
    pub tcp_ports_no_banners: bool,
    #[clap(long, default_value_t = 1000)]
    /// Timeout in seconds for banner grabbing.
    pub tcp_ports_banner_timeout: u64,
    #[clap(long, default_value = "80, 8080, 8081, 8888")]
    /// Comma separated list of ports for HTTP grabbing.
    pub tcp_ports_http: String,
    #[clap(long, default_value = "443, 8443")]
    /// Comma separated list of ports for HTTPS grabbing.
    pub tcp_ports_https: String,
    #[clap(long, default_value = "server, x-powered-by, location, content-type")]
    /// Comma separated list lowercase header names for HTTP/HTTPS grabbing.
    pub tcp_ports_http_headers: String,
}
