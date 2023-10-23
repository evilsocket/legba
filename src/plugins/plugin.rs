use std::time::Duration;

use async_trait::async_trait;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::Options;

#[async_trait]
pub(crate) trait Plugin: Sync + Send {
    fn description(&self) -> &'static str;

    fn single_credential(&self) -> bool {
        false
    }

    fn setup(&mut self, options: &Options) -> Result<(), Error>;

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error>;
}
