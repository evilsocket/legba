use clap::{ArgMatches, CommandFactory as _, Parser, parser::ValueSource};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{creds, session};

// NOTE: normally we'd be using clap subcommands, but this approach allows us more flexibility
// for plugins registered at runtime, aliases (like ssh/sftp) and so on.

// TODO: refactor with subcommands?

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[clap(version, arg_required_else_help(true))]
pub(crate) struct Options {
    #[clap(short = 'L', long, default_value_t = false)]
    /// List all available protocol plugins.
    pub list_plugins: bool,
    /// Protocol plugin to use, run with --list-plugins for a list of all available plugins.
    pub plugin: Option<String>,
    #[clap(short = 'R', long)]
    /// Load a recipe from this YAML file.
    pub recipe: Option<String>,

    /// Single target host, url or IP address, IP range, CIDR, @filename or comma separated combination of them.
    #[clap(short = 'T', long)]
    pub target: Option<String>,

    /// Enable the REST API and bind it to the specified address:port.
    #[clap(long)]
    pub api: Option<String>,
    /// Use a more restrictive CORS policy by only allowing requests from the specified origin.
    #[clap(long, default_value = "127.0.0.1")]
    pub api_allowed_origin: String,

    /// Enable the MCP server and bind it to the specified address:port. Use stdio to use standard input/output instead of SSE.
    #[clap(long)]
    pub mcp: Option<String>,

    /// Constant, filename, glob expression as @/some/path/*.txt, permutations as #min-max:charset / #min-max or range as [min-max] / [n, n, n]
    #[clap(short = 'U', long, visible_alias = "payloads")]
    pub username: Option<String>,
    /// Constant, filename, glob expression as @/some/path/*.txt or permutations as #min-max:charset / #min-max or range as [min-max] / [n, n, n]
    #[clap(short = 'P', long, visible_alias = "key")]
    pub password: Option<String>,
    /// Load username:password combinations from this file.
    #[clap(short = 'C', long)]
    pub combinations: Option<String>,
    /// Separator if using the --combinations/-C argument.
    #[clap(long, default_value = ":")]
    pub separator: String,

    /// Whether to iterate by user or by password.
    #[clap(short = 'I', long, value_enum, default_value_t = creds::IterationStrategy::User)]
    pub iterate_by: creds::IterationStrategy,

    /// Log runtime statistics and events as JSON.
    #[clap(short = 'J', long, default_value_t = false)]
    pub json: bool,

    /// Save and restore session information to this file.
    #[clap(short = 'S', long)]
    pub session: Option<String>,
    /// Save results to this file.
    #[clap(short = 'O', long)]
    pub output: Option<String>,
    /// Output file format.
    #[clap(long, value_enum, default_value_t = session::loot::OutputFormat::Text)]
    pub output_format: session::loot::OutputFormat,
    /// Connection timeout in milliseconds.
    #[clap(long, default_value_t = 10000)]
    pub timeout: u64,
    /// Number of attempts if a request fails.
    #[clap(long, default_value_t = 3)]
    pub retries: usize,
    /// Delay in milliseconds to wait before a retry.
    #[clap(long, default_value_t = 1000)]
    pub retry_time: u64,
    #[clap(long, default_value_t = false)]
    /// Exit after the first positive match is found.
    pub single_match: bool,

    /// Value for ulimit (max open file descriptors).
    #[cfg(not(windows))]
    #[clap(long, default_value_t = 10000)]
    pub ulimit: u64,

    /// Number of concurrent workers.
    #[clap(long, default_value_t = num_cpus::get())]
    pub concurrency: usize,
    /// Limit the number of requests per second.
    #[clap(long, default_value_t = 0)]
    pub rate_limit: usize,
    /// Wait time in milliseconds per login attempt.
    #[clap(short = 'W', long, default_value_t = 0)]
    pub wait: usize,
    /// Minimum number of milliseconds for random request jittering.
    #[clap(long, default_value_t = 0)]
    pub jitter_min: u64,
    /// Maximum number of milliseconds for random request jittering.
    #[clap(long, default_value_t = 0)]
    pub jitter_max: u64,
    /// Do not report statistics.
    #[clap(short = 'Q', long, default_value_t = false)]
    pub quiet: bool,

    /// Generate shell completions
    #[clap(long)]
    #[serde(skip)]
    pub generate_completions: Option<clap_complete::Shell>,

    #[clap(flatten, next_help_heading = "COMMAND (CMD)")]
    pub cmd: crate::plugins::cmd::options::Options,
    #[cfg(feature = "amqp")]
    #[clap(flatten, next_help_heading = "AMQP")]
    pub amqp: crate::plugins::amqp::options::Options,
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
    #[cfg(feature = "snmp")]
    #[clap(flatten, next_help_heading = "SNMP")]
    pub snmp: crate::plugins::snmp::options::Options,
    #[cfg(feature = "socks5")]
    #[clap(flatten, next_help_heading = "SOCKS5")]
    pub socks5: crate::plugins::socks5::options::Options,
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
    #[cfg(feature = "mqtt")]
    #[clap(flatten, next_help_heading = "MQTT")]
    pub mqtt: crate::plugins::mqtt::options::Options,
    #[cfg(feature = "redis")]
    #[clap(flatten, next_help_heading = "REDIS")]
    pub redis: crate::plugins::redis::options::Options,
    #[cfg(feature = "port_scanner")]
    #[clap(flatten, next_help_heading = "PORT SCANNER")]
    pub port_scanner: crate::plugins::port_scanner::options::Options,
    #[cfg(feature = "irc")]
    #[clap(flatten, next_help_heading = "IRC")]
    pub irc: crate::plugins::irc::options::Options,
}

fn update_field_by_name(
    options: &mut Options,
    field_name: &str,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize current options to JSON
    let mut options_value: Value = serde_json::to_value(&options)?;

    // Get the object map
    if let Value::Object(ref mut map) = options_value {
        // Get the current value to determine its type
        if let Some(current_value) = map.get(field_name) {
            match current_value {
                Value::Null | Value::String(_) => {
                    if let Some(val) = matches.get_one::<String>(field_name) {
                        log::debug!("updating field: {:?} = {:?}", &field_name, &val);
                        map.insert(field_name.to_string(), Value::String(val.clone()));
                    } else {
                        return Err(format!("Missing value for field: {:?}", &field_name).into());
                    }
                }
                Value::Number(_) => {
                    if let Some(val) = matches.get_one::<usize>(field_name) {
                        log::debug!("updating field: {:?} = {:?}", &field_name, &val);
                        map.insert(field_name.to_string(), Value::Number((*val).into()));
                    } else {
                        return Err(format!("Missing value for field: {:?}", &field_name).into());
                    }
                }
                Value::Bool(_) => {
                    if let Some(val) = matches.get_one::<bool>(field_name) {
                        log::debug!("updating field: {:?} = {:?}", &field_name, &val);
                        map.insert(field_name.to_string(), Value::Bool(*val));
                    } else {
                        return Err(format!("Missing value for field: {:?}", &field_name).into());
                    }
                }
                _ => {
                    return Err(format!(
                        "Unknown or unsupported type for field: {:?} (current_value={:?})",
                        &field_name, &current_value
                    )
                    .into());
                }
            }
        }
    } else {
        return Err(format!(
            "Invalid options structure: expected JSON object for field: {:?}",
            &field_name
        )
        .into());
    }

    // Deserialize back to Options
    *options = serde_json::from_value(options_value)?;

    Ok(())
}

pub(crate) fn update_selectively(
    options: &mut Options,
    argv: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Options::command();
    let matches = cmd.clone().try_get_matches_from(argv)?;

    // Update only the fields that were explicitly provided
    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if matches.contains_id(id)
            && matches.value_source(id) == Some(ValueSource::CommandLine)
            && id != "plugin"
            && id != "P"
            && id != "R"
            && id != "recipe"
        {
            update_field_by_name(options, id, &matches)?;
        }
    }

    Ok(())
}
