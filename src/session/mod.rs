use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use crate::creds::{Combinator, Expression};
use crate::Options;

pub(crate) mod loot;

use crate::utils::{parse_multiple_targets, parse_target};
pub(crate) use crate::Credentials;
pub(crate) use loot::Loot;

use std::sync::{Arc, Mutex};
use std::time;

pub(crate) type Error = String;

async fn periodic_saver(session: Arc<Session>, persistent: bool) {
    let one_sec = time::Duration::from_millis(1000);
    let mut last_done: usize = 0;

    while !session.is_stop() {
        std::thread::sleep(one_sec);

        // compute number of attempts per second
        let new_done = session.get_done();
        let speed = new_done - last_done;
        last_done = new_done;

        session.set_speed(speed);

        if persistent {
            if let Err(e) = session.save() {
                log::error!("could not save session: {:?}", e);
            }
        }
    }

    if persistent {
        // update and save to the last state before exiting
        if let Err(e) = session.save() {
            log::error!("could not save session: {:?}", e);
        }
    }
}

#[derive(Debug)]
struct Runtime {
    stop: AtomicBool,
    creds_tx: async_channel::Sender<Credentials>,
    creds_rx: async_channel::Receiver<Credentials>,
    speed: AtomicUsize,
}

impl Default for Runtime {
    fn default() -> Self {
        let (creds_tx, creds_rx) = async_channel::unbounded();
        Self {
            stop: AtomicBool::new(false),
            speed: AtomicUsize::new(0),
            creds_tx,
            creds_rx,
        }
    }
}

impl Runtime {
    fn new(concurrency: usize) -> Self {
        let (creds_tx, creds_rx) = async_channel::bounded(concurrency);
        Self {
            stop: AtomicBool::new(false),
            speed: AtomicUsize::new(0),
            creds_tx,
            creds_rx,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Session {
    pub options: Options,
    pub targets: Vec<String>,
    pub total: AtomicUsize,
    pub done: AtomicUsize,
    pub errors: AtomicUsize,
    pub results: Mutex<Vec<Loot>>,

    #[serde(skip_serializing, skip_deserializing)]
    runtime: Runtime,
}

impl Session {
    fn from_options(options: Options) -> Result<Arc<Self>, Error> {
        let targets = if let Some(single_target) = options.target.as_ref() {
            vec![single_target.to_owned()]
        } else if let Some(multi_targets) = options.multiple.as_ref() {
            parse_multiple_targets(multi_targets)?
        } else {
            return Err("no --target or --multiple argument provided".to_owned());
        };

        if targets.is_empty() {
            return Err("empty list of target(s) provided".to_owned());
        }

        // perform pre-emptive target validation
        for target in &targets {
            parse_target(target, 0)?;
        }

        let runtime = Runtime::new(options.concurrency);
        let total = AtomicUsize::new(0);
        let done = AtomicUsize::new(0);
        let errors = AtomicUsize::new(0);
        let results = Mutex::new(vec![]);

        Ok(Arc::new(Self {
            options,
            targets,
            total,
            done,
            errors,
            results,
            runtime,
        }))
    }

    fn from_disk(path: &str) -> Result<Arc<Self>, Error> {
        log::debug!("restoring session from {}", path);

        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut session: Session = serde_json::from_reader(file).map_err(|e| e.to_string())?;

        session.runtime = Runtime::new(session.options.concurrency);

        Ok(Arc::new(session))
    }

    pub fn new(options: Options) -> Result<Arc<Self>, Error> {
        let mut persistent = false;
        // if a session file has been specified
        let session = if let Some(path) = &options.session {
            persistent = true;
            // if it already exists
            if Path::new(path).exists() {
                // restore from disk
                Self::from_disk(path)?
            } else {
                // create new with persistency
                Self::from_options(options)?
            }
        } else {
            // create new without persistency
            Self::from_options(options)?
        };

        // set ctrl-c handler
        let le_session = session.clone();
        ctrlc::set_handler(move || {
            log::info!("stopping ...");
            le_session.set_stop();
        })
        .expect("error setting ctrl-c handler");

        tokio::task::spawn(periodic_saver(session.clone(), persistent));

        Ok(session)
    }

    pub fn is_stop(&self) -> bool {
        self.runtime.stop.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_done(&self) -> bool {
        self.get_done() >= self.get_total()
    }

    pub fn is_finished(&self) -> bool {
        self.is_done() || self.is_stop()
    }

    pub fn set_stop(&self) {
        self.runtime.stop.store(true, Ordering::SeqCst);
    }

    pub fn inc_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_errors(&self) -> usize {
        self.errors.load(Ordering::Relaxed)
    }

    pub fn inc_done(&self) {
        self.done.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_done(&self) -> usize {
        self.done.load(Ordering::Relaxed)
    }

    pub fn set_total(&self, value: usize) {
        self.total.store(value, Ordering::Relaxed);
    }

    pub fn get_total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    pub fn set_speed(&self, rps: usize) {
        self.runtime.speed.store(rps, Ordering::Relaxed);
    }

    pub fn get_speed(&self) -> usize {
        self.runtime.speed.load(Ordering::Relaxed)
    }

    pub fn combinations(
        &self,
        override_payload: Option<Expression>,
        single: bool,
    ) -> Result<Combinator, Error> {
        let combinator = if let Some(expr) = override_payload {
            Combinator::from_plugin_override(
                &self.targets,
                expr,
                self.get_done(),
                self.options.clone(),
            )?
        } else {
            Combinator::from_options(&self.targets, self.options.clone(), self.get_done(), single)?
        };

        self.set_total(combinator.search_space_size());

        if single {
            log::info!("using -> {}\n", combinator.username_expression());
        } else {
            log::info!("username -> {}", combinator.username_expression());
            log::info!("password -> {}\n", combinator.password_expression());
        }

        Ok(combinator)
    }

    pub async fn dispatch_new_credentials(&self, creds: Credentials) -> Result<(), Error> {
        self.runtime
            .creds_tx
            .send(creds)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn recv_new_credentials(&self) -> Result<Credentials, Error> {
        self.runtime
            .creds_rx
            .recv()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn add_loot(&self, loot: Loot) -> Result<(), Error> {
        // append to loot vector
        if let Ok(mut results) = self.results.lock() {
            if !results.contains(&loot) {
                results.push(loot.clone());

                // report credentials to screen
                log::info!("{}", &loot);

                // check if we have to output to file
                if let Some(path) = &self.options.output {
                    if let Err(e) = loot.append_to_file(path, &self.options.output_format) {
                        log::error!("could not write to {}: {:?}", &path, e);
                    }
                }

                // if we only need one match, stop
                if !loot.is_partial() && self.options.single_match {
                    self.set_stop();
                }

                // save session if needed
                return self.save();
            }
        } else {
            return Err("could not lock session results".to_owned());
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), Error> {
        if let Some(path) = self.options.session.as_ref() {
            log::debug!("saving session to {}", path);
            let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
            return fs::write(path, json).map_err(|e| e.to_string());
        }
        Ok(())
    }
}
