use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "")]
    /// Domain name.
    pub rdp_domain: String,
    #[clap(long, default_value_t = false)]
    /// Use a NTLM hash instead of a password.
    pub rdp_ntlm: bool,
    #[clap(long, default_value_t = false)]
    /// Restricted admin mode.
    pub rdp_admin_mode: bool,
    #[clap(long, default_value_t = false)]
    /// AutoLogon mode in case of SSL negotiation.
    pub rdp_auto_logon: bool,
}
