use std::time::Duration;

use async_trait::async_trait;

use crate::Options;
use crate::creds::{Credentials, Expression};
use crate::session::{Error, Loot};

/// What type of payload is consumed by a plugin.
pub(crate) enum PayloadStrategy {
    /// Single payload like for dns, tcp.port, etc
    Single,
    /// Standard double payload.
    UsernamePassword,
}

impl std::fmt::Display for PayloadStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PayloadStrategy::Single => "single",
                PayloadStrategy::UsernamePassword => "username_and_password",
            }
        )
    }
}

#[async_trait]
pub(crate) trait Plugin: Sync + Send {
    // return the description for this plugin
    fn description(&self) -> &'static str;

    // plugin payload strategy
    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::UsernamePassword
    }

    // single credential plugins can override this method to return their own payload expression
    fn override_payload(&self) -> Option<Expression> {
        None
    }

    // configure the plugin initial state
    fn setup(&mut self, options: &Options) -> Result<(), Error>;

    // perform a plugin step with the given credentials and timeout
    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error>;
}
