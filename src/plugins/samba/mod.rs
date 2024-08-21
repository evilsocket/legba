use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

use async_trait::async_trait;
use pavao::{SmbClient, SmbCredentials, SmbDirentType, SmbOptions};
use tokio::sync::Mutex;

use crate::creds::Credentials;
use crate::session::{Error, Loot};
use crate::Plugin;
use crate::{utils, Options};

pub(crate) mod options;

static SHARE_CACHE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PAVAO_LOCK: Mutex<()> = Mutex::const_new(());

super::manager::register_plugin! {
    "smb" => SMB::new()
}

#[derive(Clone)]
pub(crate) struct SMB {
    share: Option<String>,
    workgroup: String,
}

impl SMB {
    pub fn new() -> Self {
        SMB {
            share: None,
            workgroup: String::default(),
        }
    }

    fn get_samba_client(
        &self,
        server: &str,
        workgroup: &str,
        share: &str,
        username: &str,
        password: &str,
    ) -> Result<SmbClient, Error> {
        SmbClient::new(
            SmbCredentials::default()
                .server(server)
                .share(share)
                .username(username)
                .password(password)
                .workgroup(workgroup),
            SmbOptions::default()
                .no_auto_anonymous_login(false)
                .one_share_per_server(true),
        )
        .map_err(|e| format!("error creating client for {}: {}", share, e))
    }

    async fn get_share_for(&self, target: &str) -> Result<String, Error> {
        if let Some(share) = self.share.as_ref() {
            // return from arguments
            return Ok(share.clone());
        }

        let mut guard = SHARE_CACHE.lock().await;
        if let Some(share) = guard.get(target) {
            // return from cache
            return Ok(share.clone());
        }

        // get from listing
        log::info!("searching private share for {} ...", target);

        let server = format!("smb://{}", target);
        let root_cli = self.get_samba_client(&server, &self.workgroup, "", "", "")?;

        if let Ok(entries) = root_cli.list_dir("") {
            for entry in entries {
                match entry.get_type() {
                    SmbDirentType::FileShare | SmbDirentType::Dir => {
                        let share = format!("/{}", entry.name());
                        // if share is private we expect an error
                        let sub_cli =
                            self.get_samba_client(&server, &self.workgroup, &share, "", "")?;
                        let listing = sub_cli.list_dir("");
                        if listing.is_err() {
                            log::info!("{}{} found", &server, &share);
                            // found a private share, update the cache and return.
                            guard.insert(target.to_owned(), share.clone());
                            return Ok(share);
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(format!(
            "could not find private share for {}, provide one with --smb-share",
            target
        ))
    }
}

#[async_trait]
impl Plugin for SMB {
    fn description(&self) -> &'static str {
        "Samba password authentication."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.share = opts.smb.smb_share.clone();
        self.workgroup = opts.smb.smb_workgroup.clone();
        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let address = utils::parse_target_address(&creds.target, 445)?;
        let server = format!("smb://{}", &address);
        let share = tokio::time::timeout(timeout, self.get_share_for(&address))
            .await
            .map_err(|e: tokio::time::error::Elapsed| e.to_string())?
            .map_err(|e| e.to_string())?;

        // HACK: pavao doesn't seem to be thread safe, so we need to acquire this lock here.
        // Sadly this decreases performances, but it appears that there are no alternatives
        // for rust :/
        let _guard = PAVAO_LOCK.lock().await;
        let client = self.get_samba_client(
            &server,
            &self.workgroup,
            &share,
            &creds.username,
            &creds.password,
        )?;

        return if client.list_dir("/").is_ok() {
            Ok(Some(vec![Loot::new(
                "smb",
                &address,
                [
                    ("username".to_owned(), creds.username.to_owned()),
                    ("password".to_owned(), creds.password.to_owned()),
                ],
            )]))
        } else {
            Ok(None)
        };
    }
}
