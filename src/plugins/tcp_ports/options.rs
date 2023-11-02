use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Options {
    #[clap(long, default_value = "1-65535")]
    /// Range or comma separated values of integer port numbers to scan.
    pub tcp_ports: String,
}
