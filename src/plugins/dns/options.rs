use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long)]
    /// Comma separatd list of DNS resolvers to use instead of the system one.
    pub dns_resolvers: Option<String>,
    #[clap(long, default_value_t = 53)]
    /// Resolver(s) port.
    pub dns_port: u16,
    #[clap(long, default_value_t = 1)]
    /// Number of retries after lookup failure before giving up.
    pub dns_attempts: usize,
    #[clap(long)]
    /// Perform ip to hostname lookup.
    pub dns_ip_lookup: bool,
    #[clap(long, default_value_t = 10)]
    /// If more than this amount of sequential dns resolutions point to the same ip, add that ip to an ignore list.
    pub dns_max_positives: usize,
    #[clap(long, default_value_t = false)]
    /// Do not fetch HTTPS certificates for new domains.
    pub dns_no_https: bool,
}
