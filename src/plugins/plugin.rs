use std::time::Duration;

use async_trait::async_trait;

use crate::creds::{Credentials, Expression};
use crate::session::{Error, Loot};
use crate::Options;

#[async_trait]
pub(crate) trait Plugin: Sync + Send {
    // return the description for this plugin
    fn description(&self) -> &'static str;

    // plugins that require a single payload instead of a username+password combination should
    // override this method and return true
    fn single_credential(&self) -> bool {
        false
    }

    // single credential plugins can override this method to return their own payload expression
    fn override_payload(&self) -> Option<Expression> {
        None
    }

    // configure the plugin initial state
    fn setup(&mut self, options: &Options) -> Result<(), Error>;

    // perform a plugin step with the given credentials and timeout
    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error>;
}
