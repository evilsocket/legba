use std::{
    collections::HashMap,
    process::Stdio,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use actix_web::Result;
use clap::Parser;
use serde::Serialize;
use tokio::{io::AsyncBufReadExt, sync::RwLock};

use crate::{session::Error, Options};

pub(crate) type SharedState = Arc<RwLock<State>>;

fn get_current_exe() -> Result<String, Error> {
    // TODO: handle errors
    Ok(std::env::current_exe()
        .map_err(|e| e.to_string())?
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned())
}

#[derive(Serialize)]
pub(crate) struct Completion {
    completed_at: SystemTime,
    exit_code: i32,
    error: Option<Error>,
}

impl Completion {
    fn with_status(exit_code: i32) -> Self {
        Self {
            completed_at: SystemTime::now(),
            exit_code,
            error: None,
        }
    }

    fn with_error(error: Error) -> Self {
        Self {
            completed_at: SystemTime::now(),
            exit_code: -1,
            error: Some(error),
        }
    }
}

async fn pipe_reader_to_writer<R: AsyncBufReadExt + Unpin>(
    reader: R,
    output: Arc<Mutex<Vec<String>>>,
) {
    let mut lines = reader.lines();
    while let Ok(line) = lines.next_line().await {
        match line {
            Some(line) => output.lock().unwrap().push(line.trim().to_owned()),
            None => break,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Wrapper {
    session_id: uuid::Uuid,
    process_id: u32,
    client: String,
    argv: Vec<String>,
    started_at: SystemTime,

    output: Arc<Mutex<Vec<String>>>,
    completed: Arc<Mutex<Option<Completion>>>,
}

impl Wrapper {
    pub async fn start(
        client: String,
        session_id: uuid::Uuid,
        argv: Vec<String>,
    ) -> Result<Self, Error> {
        let app = get_current_exe()?;

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
            "[{session_id}] started '{} {:?}' as process {process_id}",
            &app,
            &argv
        );

        // read stdout
        let output = Arc::new(Mutex::new(vec![]));
        let stdout_r = tokio::io::BufReader::new(child.stdout.take().unwrap());
        tokio::task::spawn(pipe_reader_to_writer(stdout_r, output.clone()));

        // read stderr
        let stderr_r = tokio::io::BufReader::new(child.stderr.take().unwrap());
        tokio::task::spawn(pipe_reader_to_writer(stderr_r, output.clone()));

        // wait for child
        let completed = Arc::new(Mutex::new(None));
        let child_completed = completed.clone();
        tokio::task::spawn(async move {
            match child.wait().await {
                Ok(code) => {
                    log::info!(
                        "[{session_id}] child process {process_id} completed with code {code}"
                    );
                    *child_completed.lock().unwrap() =
                        Some(Completion::with_status(code.code().unwrap_or(-1)));
                }
                Err(error) => {
                    log::error!(
                        "[{session_id}] child process {process_id} completed with error {error}"
                    );
                    *child_completed.lock().unwrap() =
                        Some(Completion::with_error(error.to_string()));
                }
            }
        });

        Ok(Self {
            started_at: SystemTime::now(),
            session_id,
            process_id,
            client,
            argv,
            completed,
            output,
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
pub(crate) struct State {
    sessions: HashMap<uuid::Uuid, Wrapper>,
}

impl State {
    pub fn new() -> Self {
        let sessions = HashMap::new();
        Self { sessions }
    }

    pub async fn start_new_session(
        &mut self,
        client: String,
        argv: Vec<String>,
    ) -> Result<uuid::Uuid, Error> {
        // TODO: change all errors and results to anyhow

        // validate argv
        let _ = Options::try_parse_from(&argv).map_err(|e| e.to_string())?;
        let session_id = uuid::Uuid::new_v4();

        // add to active sessions
        self.sessions.insert(
            session_id.clone(),
            Wrapper::start(client, session_id, argv).await?,
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

    pub fn active_sessions(&self) -> &HashMap<uuid::Uuid, Wrapper> {
        &self.sessions
    }

    pub fn get_session(&self, id: &uuid::Uuid) -> Option<&Wrapper> {
        self.sessions.get(id)
    }
}
