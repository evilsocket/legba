use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Options {
    #[clap(long, default_value_t = false)]
    /// Enable SSL for Redis.
    pub redis_ssl: bool,
    #[clap(long, default_value = "PING")]
    /// Redis command to execute in order to test ACL.
    pub redis_command: String,
}
