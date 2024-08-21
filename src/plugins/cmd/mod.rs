use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

pub(crate) mod options;

super::manager::register_plugin! {
    "cmd" => Command::new()
}

#[derive(Clone)]
pub(crate) struct Command {
    opts: options::Options,
}

impl Command {
    pub fn new() -> Self {
        Command {
            opts: options::Options::default(),
        }
    }

    async fn run(&self, creds: &Credentials) -> Result<std::process::Output, Error> {
        let args = shell_words::split(
            &self
                .opts
                .cmd_args
                .replace("{USERNAME}", &creds.username)
                .replace("{PASSWORD}", &creds.password)
                .replace("{TARGET}", &creds.target),
        )
        .unwrap();

        log::debug!("{} {}", &self.opts.cmd_binary, args.join(" "));

        let child = std::process::Command::new(&self.opts.cmd_binary)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        child.wait_with_output().map_err(|e| e.to_string())
    }
}

#[async_trait]
impl Plugin for Command {
    fn description(&self) -> &'static str {
        "Command execution."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.opts = opts.cmd.clone();
        if self.opts.cmd_binary.is_empty() {
            Err("no --cmd-binary provided".to_owned())
        } else {
            Ok(())
        }
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let output = tokio::time::timeout(timeout, self.run(creds))
            .await
            .map_err(|e| e.to_string())?;

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.is_empty() {
                log::debug!("STDERR: {}", stderr);
            }

            log::debug!("STDOUT: {}", &stdout);

            // check exit code first
            if out.status.code().unwrap_or(-1) == self.opts.cmd_success_exit_code {
                // then output if needed
                let ok = if let Some(pattern) = &self.opts.cmd_success_match {
                    if pattern.is_empty() {
                        stdout.is_empty()
                    } else {
                        stdout.contains(pattern)
                    }
                } else {
                    true
                };

                if ok {
                    return Ok(Some(vec![Loot::new(
                        "command",
                        &creds.target,
                        [
                            ("username".to_owned(), creds.username.to_owned()),
                            ("password".to_owned(), creds.password.to_owned()),
                        ],
                    )]));
                }
            }

            return Ok(None);
        } else {
            return Err(output.err().unwrap().to_string());
        }
    }
}
