use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug, ValueEnum)]
pub(crate) enum Mode {
    Key,
    #[default]
    Password,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, value_enum, default_value_t = Mode::Password)]
    /// Authentication strategy.
    pub ssh_auth_mode: Mode,
    #[clap(long)]
    /// Optional private key passphrase for key based authentication.
    pub ssh_key_passphrase: Option<String>,
}
