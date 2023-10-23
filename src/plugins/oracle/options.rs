use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Options {
    #[clap(long, default_value = "SYSTEM")]
    /// Database name.
    pub oracle_database: String,
}
