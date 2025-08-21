use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value_t = false)]
    /// Use v5 of the MQTT protocol.
    pub mqtt_v5: bool,
    #[clap(long, default_value_t = false)]
    /// Use SSL/TLS connection (mqtts://) with certificate verification disabled.
    pub mqtt_ssl: bool,
}