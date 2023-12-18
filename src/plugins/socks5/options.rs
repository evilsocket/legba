use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "ifcfg.co")]
    /// Remote address to test the proxying for.
    pub socks5_address: String,
    #[clap(long, default_value_t = 80)]
    /// Remote port to test the proxying for.
    pub socks5_port: u16,
}
