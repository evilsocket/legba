use std::{
    collections::HashMap,
    os::unix::process::ExitStatusExt,
    process::Stdio,
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::Result;
use clap::Parser;
use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;
use serde::Serialize;
use tokio::{io::AsyncBufReadExt, sync::RwLock};

static STATS_PARSER: Lazy<Regex> = lazy_regex!(
    r"(?m)^.+tasks=(\d+)\s+mem=(.+)\stargets=(\d+)\sattempts=(\d+)\sdone=(\d+)\s\((.+)%\)(\serrors=(\d+))?\sspeed=(.+) reqs/s"
);
static LOOT_PARSER: Lazy<Regex> = lazy_regex!(r"(?m)^.+\[(.+)\]\s\(([^)]+)\)(\s<(.+)>)?\s(.+)");

use crate::{Options, session::Error, utils::parse_multiple_targets};

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

#[derive(Serialize)]
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
                // remove colors and other escape sequences
                let line = strip_ansi_escapes::strip_str(&line);
                // do not collect empty lines
                if !line.trim().is_empty() {
                    if let Some(caps) = STATS_PARSER.captures(&line) {
                        // parse as statistics
                        {
                            let mut stats_w = stats.lock().unwrap();

                            stats_w.tasks = caps.get(1).unwrap().as_str().parse().unwrap();
                            stats_w.memory = caps.get(2).unwrap().as_str().to_owned();
                            stats_w.targets = caps.get(3).unwrap().as_str().parse().unwrap();
                            stats_w.attempts = caps.get(4).unwrap().as_str().parse().unwrap();
                            stats_w.done = caps.get(5).unwrap().as_str().parse().unwrap();
                            stats_w.done_percent = caps.get(6).unwrap().as_str().parse().unwrap();
                            stats_w.errors = if let Some(errs) = caps.get(8) {
                                errs.as_str().parse().unwrap()
                            } else {
                                0
                            };
                            stats_w.reqs_per_sec = caps.get(9).unwrap().as_str().parse().unwrap();
                        }
                    } else if let Some(caps) = LOOT_PARSER.captures(&line) {
                        // parse as loot
                        loot.lock().unwrap().push(Loot {
                            found_at: caps.get(1).unwrap().as_str().to_owned(),
                            plugin: caps.get(2).unwrap().as_str().to_owned(),
                            target: if let Some(t) = caps.get(4) {
                                Some(t.as_str().to_owned())
                            } else {
                                None
                            },
                            data: caps.get(5).unwrap().as_str().to_owned(),
                        });
                    } else {
                        // add as raw output
                        output.lock().unwrap().push(line.trim().to_owned());
                    }
                }
            }
            None => break,
        }
    }
}

#[derive(Default, Serialize)]
pub(crate) struct Loot {
    found_at: String,
    plugin: String,
    target: Option<String>,
    data: String,
}

#[derive(Default, Serialize)]
pub(crate) struct Statistics {
    tasks: usize,
    memory: String,
    targets: usize,
    attempts: usize,
    errors: usize,
    done: usize,
    done_percent: f32,
    reqs_per_sec: usize,
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
                    let signal = code.signal().unwrap_or(0);
                    // ok or terminated
                    if code.success() || signal == 15 {
                        log::info!("[{id}] child process {process_id} completed with code {code}");
                        *child_completed.lock().unwrap() =
                            Some(Completion::with_status(code.code().unwrap_or(-1)));
                    } else {
                        log::error!(
                            "[{id}] child process {process_id} completed with code {code} (signal {:?})",
                            code.signal()
                        );
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
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(self.process_id as nix::libc::pid_t),
            nix::sys::signal::Signal::SIGTERM,
        )
        .map_err(|e| e.to_string())
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

    pub async fn start_new_session(
        &mut self,
        client: String,
        argv: Vec<String>,
    ) -> Result<uuid::Uuid, Error> {
        // TODO: change all errors and results to anyhow

        // validate argv
        let opts = Options::try_parse_from(&argv).map_err(|e| e.to_string())?;
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
            session_id.clone(),
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
}
