use clap::Parser;
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_PORTS: &str = "[1-65535]";

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(
        long,
        default_value = DEFAULT_PORTS
    )]
    /// Range or comma separated values of integer port numbers to scan.
    pub port_scanner_ports: String,
    /// Do not attempt banner grabbing.
    #[clap(long, default_value_t = false)]
    pub port_scanner_no_banners: bool,
    /// Do not perform UDP scan.
    #[clap(long, default_value_t = false)]
    pub port_scanner_no_udp: bool,
    /// Do not perform TCP scan.
    #[clap(long, default_value_t = false)]
    pub port_scanner_no_tcp: bool,
    #[clap(long, default_value_t = 1500)]
    /// Timeout in milliseconds for banner grabbing.
    pub port_scanner_banner_timeout: u64,
    #[clap(long, default_value = "80, 8080, 8081, 8888")]
    /// Comma separated list of ports for HTTP grabbing.
    pub port_scanner_http: String,
    #[clap(long, default_value = "443, 8443")]
    /// Comma separated list of ports for HTTPS grabbing.
    pub port_scanner_https: String,
    #[clap(long, default_value = "server, x-powered-by, location, content-type")]
    /// Comma separated list lowercase header names for HTTP/HTTPS grabbing.
    pub port_scanner_http_headers: String,
}
