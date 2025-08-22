use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long)]
    /// Specify a single OID to read, if not specified the entire SNMP tree is walked.
    pub snmp_oid: Option<String>,
    /// Specify a maximum number of OIDs to walk. Set to 0 to walk the entire SNMP tree (it may take a long time).
    #[clap(long, default_value_t = 0xff)]
    pub snmp_max: usize,
}
