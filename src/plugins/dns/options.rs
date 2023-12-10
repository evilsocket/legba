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
}
