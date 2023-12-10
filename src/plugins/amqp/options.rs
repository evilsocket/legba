use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value_t = false)]
    /// Enable SSL for AMQP.
    pub amqp_ssl: bool,
}
