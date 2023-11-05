use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Options {
    #[clap(long, default_value_t = String::from("legba"))]
    // MQTT client id
    pub mqtt_client_id: String,
}
