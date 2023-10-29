use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Options {
    #[clap(long, default_value_t = false)]
    /// Enable SSL for Redis.
    pub redis_ssl: bool,
}
