use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    /// SMTP authentication mechanism, can be PLAIN (RFC4616), LOGIN (obsolete but needed for some providers like office365) or XOAUTH2.
    #[clap(long, default_value = "PLAIN")]
    pub smtp_mechanism: String,
}
