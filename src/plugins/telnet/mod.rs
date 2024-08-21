use std::time::Duration;

use async_trait::async_trait;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

pub(crate) mod options;

super::manager::register_plugin! {
    "telnet" => Telnet::new()
}

#[derive(Clone)]
pub(crate) struct Telnet {
    user_prompt: String,
    pass_prompt: String,
    shell_prompt: String,
}

impl Telnet {
    pub fn new() -> Self {
        Telnet {
            user_prompt: String::new(),
            pass_prompt: String::new(),
            shell_prompt: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for Telnet {
    fn description(&self) -> &'static str {
        "Telnet password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.user_prompt.clone_from(&opts.telnet.telnet_user_prompt);
        self.pass_prompt.clone_from(&opts.telnet.telnet_pass_prompt);
        self.shell_prompt.clone_from(&opts.telnet.telnet_prompt);
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 23)?;
        let mut client = mini_telnet::Telnet::builder()
            .connect_timeout(Duration::from_secs(10))
            .login_prompt(&self.user_prompt, &self.pass_prompt)
            .prompt(&self.shell_prompt)
            .timeout(timeout)
            .connect(&address)
            .await
            .map_err(|e| e.to_string())?;

        if client.login(&creds.username, &creds.password).await.is_ok() {
            Ok(Some(vec![Loot::new(
                "telnet",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
        } else {
            Ok(None)
        }
    }
}
