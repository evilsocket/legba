use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "WORKGROUP", help_heading = "SMB")]
    /// Samba workgroup name.
    pub smb_workgroup: String,
    #[clap(long, default_value = "IPC$", help_heading = "SMB")]
    /// Explicitly set Samba private share to test.
    pub smb_share: Option<String>,
}
