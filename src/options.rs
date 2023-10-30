use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::session;

// NOTE: normally we'd be using clap subcommands, but this approach allows us more flexibility
// for plugins registered at runtime, aliases (like ssh/sftp) and so on.

// TODO: refactor with subcommands?

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[clap(version, arg_required_else_help(true))]
pub(crate) struct Options {
    #[clap(long, default_value_t = false)]
    /// List all available protocol plugins.
    pub list_plugins: bool,

    /// Protocol plugin to use, run with --list-plugins for a list of all available plugins.
    pub plugin: Option<String>,
    /// Target host, url or IP address.
    #[clap(short, long)]
    pub target: Option<String>,
    /// Constant, filename, glob expression as @/some/path/*.txt or range as #min-max:charset / #min-max
    #[clap(long, visible_alias = "data")]
    pub username: Option<String>,
    /// Constant, filename, glob expression as @/some/path/*.txt or range as #min-max:charset / #min-max
    #[clap(long, visible_alias = "key")]
    pub password: Option<String>,

    /// Save and restore session information to this file.
    #[clap(short, long)]
    pub session: Option<String>,
    /// Save results to this file.
    #[clap(short, long)]
    pub output: Option<String>,
    /// Output file format.
    #[clap(long, value_enum, default_value_t = session::loot::OutputFormat::Text)]
    pub output_format: session::loot::OutputFormat,
    /// Connection timeout in milliseconds.
    #[clap(long, default_value_t = 1000)]
    pub timeout: u64,
    /// Number of attempts if a request fails.
    #[clap(long, default_value_t = 10)]
    pub retries: usize,
    /// Delay in milliseconds to wait before a retry.
    #[clap(long, default_value_t = 1000)]
    pub retry_time: u64,
    #[clap(long, default_value_t = false)]
    /// Exit after the first positive match is found.
    pub single_match: bool,
    /// Value for ulimit (max open file descriptors).
    #[clap(long, default_value_t = 10000)]
    pub ulimit: u64,
    /// Number of concurrent workers.
    #[clap(long, default_value_t = num_cpus::get())]
    pub concurrency: usize,
    /// Limit the number of requests per second.
    #[clap(long, default_value_t = 0)]
    pub rate_limit: usize,
    /// Minimum number of milliseconds for random request jittering.
    #[clap(long, default_value_t = 0)]
    pub jitter_min: u64,
    /// Maximum number of milliseconds for random request jittering.
    #[clap(long, default_value_t = 0)]
    pub jitter_max: u64,
    /// Do not report statistics.
    #[clap(long, default_value_t = false)]
    pub quiet: bool,

    #[cfg(feature = "http")]
    #[clap(flatten, next_help_heading = "HTTP")]
    pub http: crate::plugins::http::options::Options,
    #[cfg(feature = "dns")]
    #[clap(flatten, next_help_heading = "DNS")]
    pub dns: crate::plugins::dns::options::Options,
    #[cfg(feature = "telnet")]
    #[clap(flatten, next_help_heading = "TELNET")]
    pub telnet: crate::plugins::telnet::options::Options,
    #[cfg(feature = "ssh")]
    #[clap(flatten, next_help_heading = "SSH")]
    pub ssh: crate::plugins::ssh::options::Options,
    #[cfg(feature = "smtp")]
    #[clap(flatten, next_help_heading = "SMTP")]
    pub smtp: crate::plugins::smtp::options::Options,
    #[cfg(feature = "pop3")]
    #[clap(flatten, next_help_heading = "POP3")]
    pub pop3: crate::plugins::pop3::options::Options,
    #[cfg(feature = "oracle")]
    #[clap(flatten, next_help_heading = "ORACLE")]
    pub oracle: crate::plugins::oracle::options::Options,
    #[cfg(feature = "ldap")]
    #[clap(flatten, next_help_heading = "LDAP")]
    pub ldap: crate::plugins::ldap::options::Options,
    #[cfg(feature = "kerberos")]
    #[clap(flatten, next_help_heading = "KERBEROS")]
    pub kerberos: crate::plugins::kerberos::options::Options,
    #[cfg(feature = "rdp")]
    #[clap(flatten, next_help_heading = "RDP")]
    pub rdp: crate::plugins::rdp::options::Options,
    #[cfg(feature = "redis")]
    #[clap(flatten, next_help_heading = "Redis")]
    pub redis: crate::plugins::redis::options::Options,
}
