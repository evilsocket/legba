use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "login: ")]
    /// Telnet server username login prompt string.
    pub telnet_user_prompt: String,
    #[clap(long, default_value = "Password: ")]
    /// Telnet server password login prompt string.
    pub telnet_pass_prompt: String,
    #[clap(long, default_value = ":~$ ")]
    /// Telnet server shell prompt after successful login.
    pub telnet_prompt: String,
}
