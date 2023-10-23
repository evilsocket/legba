use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("telnet", Box::new(Telnet::new()));
}

#[derive(Clone)]
pub(crate) struct Telnet {
    host: String,
    port: u16,
    address: String,
    user_prompt: String,
    pass_prompt: String,
    shell_prompt: String,
}

impl Telnet {
    pub fn new() -> Self {
        Telnet {
            host: String::new(),
            port: 23,
            address: String::new(),
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
        (self.host, self.port) = utils::parse_target(opts.target.as_ref(), 23)?;
        self.address = format!("{}:{}", &self.host, self.port);
        self.user_prompt = opts.telnet.telnet_user_prompt.clone();
        self.pass_prompt = opts.telnet.telnet_pass_prompt.clone();
        self.shell_prompt = opts.telnet.telnet_prompt.clone();
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let mut client = mini_telnet::Telnet::builder()
            .connect_timeout(Duration::from_secs(10))
            .login_prompt(&self.user_prompt, &self.pass_prompt)
            .prompt(&self.shell_prompt)
            .timeout(timeout)
            .connect(&self.address)
            .await
            .map_err(|e| e.to_string())?;

        if client.login(&creds.username, &creds.password).await.is_ok() {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else {
            Ok(None)
        }
    }
}
