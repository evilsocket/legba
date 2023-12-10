use clap::Parser;
use serde::{Deserialize, Serialize};

use super::Protocol;

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long)]
    /// Kerberos realm.
    pub kerberos_realm: Option<String>,
    #[clap(long, value_enum, default_value_t = Protocol::TCP)]
    /// Kerberos transport protocol.
    pub kerberos_protocol: Protocol,
    #[clap(long, default_value_t = false)]
    /// If targeting a Linux Kerberos5 implementation, pass this flag to preserve the realm string case.
    pub kerberos_linux: bool,
}
