use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use human_bytes::human_bytes;
use memory_stats::memory_stats;
use serde::{Deserialize, Serialize};

use crate::Options;
use crate::creds::{Combinator, Expression};

pub(crate) mod loot;
mod runtime;

use runtime::*;

pub(crate) use crate::Credentials;
use crate::utils::{parse_multiple_targets, parse_target};
pub(crate) use loot::Loot;

use std::sync::{Arc, Mutex};
use std::time;

pub(crate) type Error = String;

async fn periodic_saver(session: Arc<Session>) {
    let report_interval = time::Duration::from_millis(session.options.report_time);
    let mut last_done: usize = 0;
    let persistent = session.options.session.is_some();

    while !session.is_stop() {
        tokio::time::sleep(report_interval).await;

        // compute number of attempts per second
        let new_done = session.get_done();
        let speed = (new_done - last_done) as f64 / report_interval.as_secs_f64();
        last_done = new_done;

        session.set_speed(speed as usize);

        if persistent && let Err(e) = session.save() {
            log::error!("could not save session: {:?}", e);
        }
    }

    // update and save to the last state before exiting
    if persistent && let Err(e) = session.save() {
        log::error!("could not save session: {:?}", e);
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Statistics {
    tasks: usize,
    memory: f64,
    targets: usize,
    attempts: usize,
    done: usize,
    done_percent: f32,
    errors: usize,
    reqs_per_sec: usize,
}

impl Statistics {
    pub fn to_text(&self) -> String {
        if self.errors > 0 {
            format!(
                "tasks={} mem={} targets={} attempts={} done={} ({:.2?}%) errors={} speed={:.2?} reqs/s",
                self.tasks,
                human_bytes(self.memory),
                self.targets,
                self.attempts,
                self.done,
                self.done_percent,
                self.errors,
                self.reqs_per_sec,
            )
        } else {
            format!(
                "tasks={} mem={} targets={} attempts={} done={} ({:.2?}%) speed={:.2?} reqs/s",
                self.tasks,
                human_bytes(self.memory),
                self.targets,
                self.attempts,
                self.done,
                self.done_percent,
                self.reqs_per_sec,
            )
        }
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    pub fn update_from_json(&mut self, json: &str) -> Result<(), Error> {
        *self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(())
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
        let targets = if let Some(target) = options.target.as_ref() {
            parse_multiple_targets(target)?
        } else {
            return Err("no --target/-T argument provided".to_owned());
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

    fn from_disk(path: &str, options: Options) -> Result<Arc<Self>, Error> {
        if Path::new(path).exists() {
            log::info!("restoring session from {}", path);

            let file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut session: Session = serde_json::from_reader(file).map_err(|e| e.to_string())?;

            session.runtime = Runtime::new(session.options.concurrency);

            Ok(Arc::new(session))
        } else {
            Self::from_options(options)
        }
    }

    pub fn new(options: Options) -> Result<Arc<Self>, Error> {
        // if a session file has been specified
        let session = if let Some(path) = options.session.as_ref() {
            // load from disk if file exists, or from options and save to disk
            Self::from_disk(path, options.clone())?
        } else {
            // create new without persistency
            Self::from_options(options)?
        };

        let num_targets = session.targets.len();
        log::info!(
            "target{}: {}",
            if num_targets > 1 {
                format!("s ({})", num_targets)
            } else {
                "".to_owned()
            },
            session.options.target.as_ref().unwrap()
        );

        // set ctrl-c handler
        let le_session = session.clone();
        ctrlc::set_handler(move || {
            // avoid triggering this if ctrl-c has been already triggered
            if !le_session.is_stop() {
                log::info!("stopping ...");
                le_session.set_stop();
            }
        })
        .expect("error setting ctrl-c handler");

        tokio::task::spawn(periodic_saver(session.clone()));

        Ok(session)
    }

    #[cfg(test)]
    pub fn new_for_tests(options: Options) -> Result<Arc<Self>, Error> {
        // if a session file has been specified
        let session = if let Some(path) = options.session.as_ref() {
            // load from disk if file exists, or from options and save to disk
            Self::from_disk(path, options.clone())?
        } else {
            // create new without persistency
            Self::from_options(options)?
        };

        // Don't set ctrl-c handler in tests and don't spawn periodic saver
        Ok(session)
    }

    pub fn is_stop(&self) -> bool {
        self.runtime.is_stop()
    }

    pub fn set_stop(&self) {
        self.runtime.set_stop()
    }

    pub fn set_speed(&self, rps: usize) {
        self.runtime.set_speed(rps);
    }

    pub fn get_speed(&self) -> usize {
        self.runtime.get_speed()
    }

    pub async fn send_credentials(&self, creds: Credentials) -> Result<(), Error> {
        self.runtime.send_credentials(creds).await
    }

    pub async fn recv_credentials(&self) -> Result<Credentials, Error> {
        self.runtime.recv_credentials().await
    }

    pub fn is_done(&self) -> bool {
        self.get_done() >= self.get_total()
    }

    pub fn is_finished(&self) -> bool {
        self.is_done() || self.is_stop()
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

    pub fn combinations(
        &self,
        override_payload: Option<Expression>,
        single: bool,
    ) -> Result<Combinator, Error> {
        let combinator = Combinator::create(
            &self.targets,
            self.options.clone(),
            self.get_done(),
            single,
            override_payload,
        )?;

        self.set_total(combinator.search_space_size());

        if single {
            log::info!("using -> {}\n", combinator.username_expression());
        } else {
            log::info!("username -> {}", combinator.username_expression());
            log::info!("password -> {}\n", combinator.password_expression());
        }

        Ok(combinator)
    }

    pub async fn add_loot(&self, loot: Loot) -> Result<(), Error> {
        // append to loot vector
        if let Ok(mut results) = self.results.lock() {
            if !results.contains(&loot) {
                results.push(loot.clone());

                // report credentials to screen
                if self.options.json {
                    println!("{}", loot.to_json().unwrap());
                } else {
                    log::info!("{}", &loot);
                }

                // check if we have to output to file
                if let Some(path) = &self.options.output
                    && let Err(e) = loot.append_to_file(path, &self.options.output_format)
                {
                    log::error!("could not write to {}: {:?}", &path, e);
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

    pub async fn report_runtime_statistics(&self) {
        let report_interval = time::Duration::from_millis(self.options.report_time);
        while !self.is_stop() {
            tokio::time::sleep(report_interval).await;

            let total = self.get_total();
            let done = self.get_done();
            let perc = (done as f32 / total as f32) * 100.0;
            let errors = self.get_errors();
            let speed: usize = self.get_speed();
            let memory = if let Some(usage) = memory_stats() {
                usage.physical_mem
            } else {
                log::error!("couldn't get the current memory usage");
                0
            };

            let stats = Statistics {
                tasks: self.options.concurrency,
                memory: memory as f64,
                targets: self.targets.len(),
                attempts: total,
                done,
                done_percent: perc,
                errors,
                reqs_per_sec: speed,
            };

            if self.options.json {
                println!("{}", stats.to_json().unwrap());
            } else {
                log::info!("{}", stats.to_text());
            }
        }
    }
}
