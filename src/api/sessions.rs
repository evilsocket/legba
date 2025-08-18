use std::{
    collections::HashMap,
    process::Stdio,
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

use actix_web::Result;
use clap::Parser;
use serde::Serialize;
use tokio::{io::AsyncBufReadExt, sync::RwLock};

use crate::{
    Options,
    session::{Error, Loot, Statistics},
    utils::parse_multiple_targets,
};

pub(crate) type SharedState = Arc<RwLock<Sessions>>;

fn get_current_exe() -> Result<String, Error> {
    // TODO: handle errors
    Ok(std::env::current_exe()
        .map_err(|e| e.to_string())?
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned())
}

#[derive(Serialize, Clone)]
pub(crate) struct Completion {
    completed_at: u64,
    exit_code: i32,
    error: Option<Error>,
}

impl Completion {
    fn with_status(exit_code: i32) -> Self {
        Self {
            completed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            exit_code,
            error: None,
        }
    }

    fn with_error(error: Error) -> Self {
        Self {
            completed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            exit_code: -1,
            error: Some(error),
        }
    }
}

async fn pipe_reader_to_writer<R: AsyncBufReadExt + Unpin>(
    reader: R,
    output: Arc<Mutex<Vec<String>>>,
    stats: Arc<Mutex<Statistics>>,
    loot: Arc<Mutex<Vec<Loot>>>,
) {
    let mut lines = reader.lines();
    while let Ok(line) = lines.next_line().await {
        match line {
            Some(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if line.starts_with('{') {
                    // json loot or runtime statistics
                    if line.contains("found_at") {
                        // parse as loot
                        let new_loot: Loot = serde_json::from_str(&line).unwrap();

                        log::info!(
                            "! plugin: {}, target: {:?}, data: {:?}",
                            new_loot.get_plugin(),
                            new_loot.get_target(),
                            new_loot.get_data()
                        );

                        loot.lock().unwrap().push(new_loot);
                    } else {
                        // parse as runtime statistics
                        stats.lock().unwrap().update_from_json(&line).unwrap();
                    }
                } else {
                    // anything else
                    // remove colors and other escape sequences
                    let line = strip_ansi_escapes::strip_str(&line);
                    // add as raw output
                    output.lock().unwrap().push(line.trim().to_owned());
                }
            }
            None => break,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Session {
    id: uuid::Uuid,
    plugin_name: String,
    targets: Vec<String>,
    process_id: u32,
    client: String,
    argv: Vec<String>,
    started_at: u64,

    statistics: Arc<Mutex<Statistics>>,
    loot: Arc<Mutex<Vec<Loot>>>,
    output: Arc<Mutex<Vec<String>>>,
    completed: Arc<Mutex<Option<Completion>>>,
}

#[derive(Default, Serialize, Clone)]
pub(crate) struct LootBrief {
    target: Option<String>,
    data: String,
}

#[derive(Serialize)]
pub(crate) struct SessionBrief {
    plugin: String,
    targets: Vec<String>,
    findings: Vec<LootBrief>,
    completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SessionListing {
    id: uuid::Uuid,
    plugn: String,
    targets: Vec<String>,
    completed: bool,
    with_findings: bool,
}

impl Session {
    pub async fn start(
        client: String,
        id: uuid::Uuid,
        argv: Vec<String>,
        targets: Vec<String>,
        taken_workers: usize,
        avail_workers: Arc<AtomicU64>,
    ) -> Result<Self, Error> {
        let app = get_current_exe()?;

        let plugin_name = argv[0].to_owned();

        // https://stackoverflow.com/questions/49245907/how-to-read-subprocess-output-asynchronously
        let mut child = tokio::process::Command::new(&app)
            .args(&argv)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| e.to_string())?;

        let process_id = child.id().unwrap();

        log::info!(
            "[{id}] started '{} {:?}' as process {process_id}",
            &app,
            &argv
        );

        let loot = Arc::new(Mutex::new(vec![]));
        let statistics = Arc::new(Mutex::new(Statistics::default()));
        // read stdout
        let output = Arc::new(Mutex::new(vec![]));
        let stdout_r = tokio::io::BufReader::new(child.stdout.take().unwrap());
        tokio::task::spawn(pipe_reader_to_writer(
            stdout_r,
            output.clone(),
            statistics.clone(),
            loot.clone(),
        ));

        // read stderr
        let stderr_r = tokio::io::BufReader::new(child.stderr.take().unwrap());
        tokio::task::spawn(pipe_reader_to_writer(
            stderr_r,
            output.clone(),
            statistics.clone(),
            loot.clone(),
        ));

        // wait for child
        let completed = Arc::new(Mutex::new(None));
        let child_completed = completed.clone();
        let child_out = output.clone();
        tokio::task::spawn(async move {
            match child.wait().await {
                Ok(code) => {
                    #[cfg(unix)]
                    let signal = code.signal().unwrap_or(0);
                    #[cfg(not(unix))]
                    let signal = 0;

                    // ok or terminated
                    if code.success() || signal == 15 {
                        log::info!("[{id}] child process {process_id} completed with code {code}");
                        *child_completed.lock().unwrap() =
                            Some(Completion::with_status(code.code().unwrap_or(-1)));
                    } else {
                        #[cfg(unix)]
                        log::error!(
                            "[{id}] child process {process_id} completed with code {code} (signal {:?})",
                            code.signal()
                        );
                        #[cfg(not(unix))]
                        log::error!("[{id}] child process {process_id} completed with code {code}");
                        *child_completed.lock().unwrap() = Some(Completion::with_error(
                            child_out
                                .lock()
                                .unwrap()
                                .last()
                                .unwrap_or(&String::new())
                                .to_string(),
                        ));
                    }
                }
                Err(error) => {
                    log::error!("[{id}] child process {process_id} completed with error {error}");
                    *child_completed.lock().unwrap() =
                        Some(Completion::with_error(error.to_string()));
                }
            }

            log::info!("session {id} completed, freeing {taken_workers} workers");

            // free the workers
            avail_workers.fetch_add(taken_workers as u64, std::sync::atomic::Ordering::Relaxed);
        });

        Ok(Self {
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            id,
            plugin_name,
            targets,
            process_id,
            client,
            argv,
            completed,
            output,
            statistics,
            loot,
        })
    }

    pub fn stop(&self) -> Result<(), Error> {
        log::info!("stopping session {}", self.id);

        #[cfg(unix)]
        {
            nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(self.process_id as nix::libc::pid_t),
                nix::sys::signal::Signal::SIGTERM,
            )
            .map_err(|e| e.to_string())
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use std::process::Command;

            // On Windows, use taskkill to terminate the process
            Command::new("taskkill")
                .args(&["/PID", &self.process_id.to_string(), "/F"])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output()
                .map_err(|e| e.to_string())
                .and_then(|output| {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(String::from_utf8_lossy(&output.stderr).to_string())
                    }
                })
        }
    }

    pub fn get_listing(&self) -> SessionListing {
        SessionListing {
            id: self.id,
            plugn: self.plugin_name.clone(),
            targets: self.targets.clone(),
            completed: self.completed.lock().unwrap().is_some(),
            with_findings: !self.loot.lock().unwrap().is_empty(),
        }
    }

    pub fn get_brief(&self) -> SessionBrief {
        let loot = self.loot.lock().unwrap().clone();
        let (completed, error) = match self.completed.lock().unwrap().as_ref() {
            Some(c) => (true, c.error.clone()),
            None => (false, None),
        };

        SessionBrief {
            plugin: self.plugin_name.clone(),
            targets: self.targets.clone(),
            findings: loot
                .into_iter()
                .map(|l| LootBrief {
                    target: Some(l.get_target().to_owned()),
                    data: l
                        .get_data()
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect(),
                })
                .collect(),
            completed,
            error,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed.lock().unwrap().is_some()
    }
}

#[derive(Serialize)]
pub(crate) struct Sessions {
    sessions: HashMap<uuid::Uuid, Session>,
    available_workers: Arc<AtomicU64>,
}

impl Sessions {
    pub fn new(concurrency: usize) -> Self {
        let sessions = HashMap::new();
        let available_workers = Arc::new(AtomicU64::new(concurrency as u64));
        Self {
            sessions,
            available_workers,
        }
    }

    pub fn get_available_workers(&self) -> u64 {
        self.available_workers
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn start_new_session(
        &mut self,
        client: String,
        mut argv: Vec<String>,
    ) -> Result<uuid::Uuid, Error> {
        // TODO: change all errors and results to anyhow

        // validate argv
        let opts = Options::try_parse_from(&argv).map_err(|e| e.to_string())?;

        // force json output for easy parsing
        if !argv.contains(&"--json".to_owned()) && !argv.contains(&"-J".to_owned()) {
            argv.push("-J".to_owned());
        }

        let targets = if let Some(target) = opts.target.as_ref() {
            parse_multiple_targets(target)?
        } else {
            return Err("no --target/-T argument provided".to_owned());
        };

        let avail_workers = self
            .available_workers
            .load(std::sync::atomic::Ordering::Relaxed) as usize;
        if opts.concurrency > avail_workers {
            return Err(format!(
                "can't start new session, {avail_workers} available workers"
            ));
        }

        self.available_workers.fetch_sub(
            opts.concurrency as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        let session_id = uuid::Uuid::new_v4();

        // add to active sessions
        self.sessions.insert(
            session_id,
            Session::start(
                client,
                session_id,
                argv,
                targets,
                opts.concurrency,
                self.available_workers.clone(),
            )
            .await?,
        );

        Ok(session_id)
    }

    pub fn stop_session(&self, id: &uuid::Uuid) -> Result<(), Error> {
        let session = match self.sessions.get(id) {
            Some(s) => s,
            None => return Err(format!("session {id} not found")),
        };
        session.stop()
    }

    pub fn get_session(&self, id: &uuid::Uuid) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_sessions(&self) -> &HashMap<uuid::Uuid, Session> {
        &self.sessions
    }
}
