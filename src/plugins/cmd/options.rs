use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "")]
    /// Command binary.
    pub cmd_binary: String,

    #[clap(long, default_value = "")]
    /// Command arguments. {USERNAME}, {PASSWORD}, {TARGET} and {PORT} can be used as placeholders.
    pub cmd_args: String,

    #[clap(long, default_value_t = 0)]
    /// Process exit code to be considered as a positive match.
    pub cmd_success_exit_code: i32,

    #[clap(long)]
    /// String to look for in the process standard output to be considered as a positive match.
    pub cmd_success_match: Option<String>,
}
