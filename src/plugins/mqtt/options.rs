use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value_t = String::from("legba"))]
    /// MQTT client id.
    pub mqtt_client_id: String,
    #[clap(long, default_value_t = false)]
    /// use v5 of the MQTT protocol.
    pub mqtt_v5: bool,
}
