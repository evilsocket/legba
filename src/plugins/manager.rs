use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time;

use ahash::HashSet;
use ansi_term::Style;
use lazy_static::lazy_static;
use rand::Rng;
use std::sync::Arc;
use tokio::task;

use crate::session::{Error, Session};
use crate::Options;
use crate::Plugin;

type Inventory = BTreeMap<&'static str, Box<dyn Plugin>>;

lazy_static! {
    static ref INVENTORY: Mutex<Inventory> = Mutex::new(Inventory::new());
}

pub(crate) fn register(name: &'static str, plugin: Box<dyn Plugin>) {
    INVENTORY.lock().unwrap().insert(name, plugin);
}

pub(crate) fn list() {
    let bold = Style::new().bold();

    println!("{}\n", bold.paint("Available plugins:"));

    let max_len = INVENTORY
        .lock()
        .unwrap()
        .keys()
        .map(|k| k.len())
        .max()
        .unwrap_or(0);

    for (key, plugin) in &*INVENTORY.lock().unwrap() {
        println!(
            "  {}{} : {}",
            bold.paint(*key),
            " ".repeat(max_len - key.len()), // padding
            plugin.description()
        );
    }
}

pub(crate) fn setup(options: &Options) -> Result<&'static mut dyn Plugin, Error> {
    let plugin_name = if let Some(value) = options.plugin.as_ref() {
        value.to_string()
    } else {
        return Err("no plugin selected".to_owned());
    };

    let plugin = match INVENTORY.lock().unwrap().remove(plugin_name.as_str()) {
        Some(p) => Box::leak(p), // makes the plugin &'static
        None => return Err(format!("{} is not a valid plugin name, run with --list-plugins to see the list of available plugins", plugin_name)),
    };

    let target = if let Some(value) = options.target.as_ref() {
        value.to_string()
    } else if let Some(multi) = options.multiple.as_ref() {
        format!("(multiple) {}", multi)
    } else {
        return Err("no --target or --multiple argument selected".to_owned());
    };

    log::info!("targeting {}", target);

    plugin.setup(options)?;

    Ok(plugin)
}

pub(crate) async fn run(
    plugin: &'static mut dyn Plugin,
    session: Arc<Session>,
) -> Result<(), Error> {
    let single = plugin.single_credential();
    let override_payload = plugin.override_payload();

    // spawn worker threads
    for _ in 0..session.options.concurrency {
        task::spawn(worker(plugin, session.clone()));
    }

    // loop credentials for this session
    for creds in session.combinations(override_payload, single)? {
        // exit on ctrl-c if we have to, otherwise send the new credentials to the workers
        if session.is_stop() {
            log::debug!("exiting loop");
            return Ok(());
        } else if let Err(e) = session.dispatch_new_credentials(creds).await {
            log::error!("{}", e.to_string());
        }
    }

    Ok(())
}

async fn worker(plugin: &dyn Plugin, session: Arc<Session>) {
    log::debug!("worker started");

    let timeout = time::Duration::from_millis(session.options.timeout);
    let retry_time: time::Duration = time::Duration::from_millis(session.options.retry_time);
    let mut unreachables: HashSet<String> = HashSet::default();

    while let Ok(creds) = session.recv_new_credentials().await {
        if session.is_stop() {
            log::debug!("exiting worker");
            break;
        }

        let mut errors = 0;
        let mut attempt = 0;

        while attempt < session.options.retries && !session.is_stop() {
            // perform random jitter if needed
            if session.options.jitter_max > 0 {
                let ms = rand::thread_rng()
                    .gen_range(session.options.jitter_min..=session.options.jitter_max);
                if ms > 0 {
                    log::debug!("jitter of {} ms", ms);
                    std::thread::sleep(time::Duration::from_millis(ms));
                }
            }

            attempt += 1;

            // skip attempt if we had enough failures from this specific target
            if !unreachables.contains(&creds.target) {
                match plugin.attempt(&creds, timeout).await {
                    Err(err) => {
                        errors += 1;
                        if attempt < session.options.retries {
                            log::debug!(
                                "[{}] attempt {}/{}: {}",
                                &creds.target,
                                attempt,
                                session.options.retries,
                                err
                            );
                            std::thread::sleep(retry_time);
                            continue;
                        } else {
                            // add this target to the list of unreachable in order to avoi
                            // pointless attempts
                            unreachables.insert(creds.target.clone());

                            log::error!(
                                "[{}] attempt {}/{}: {}",
                                &creds.target,
                                attempt,
                                session.options.retries,
                                err
                            );
                        }
                    }
                    Ok(loot) => {
                        // do we have new loot?
                        if let Some(loot) = loot {
                            session.add_loot(loot).await.unwrap();
                        }
                    }
                };
            }

            break;
        }

        session.inc_done();
        if errors == session.options.retries {
            session.inc_errors();
            log::debug!("retries={} errors={}", session.options.retries, errors);
        }
    }

    log::debug!("worker exit");
}
