use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};
use std::time;

use ansi_term::Style;
use dashmap::DashSet;
use rand::Rng;
use std::sync::Arc;
use tokio::task;

use crate::Options;
use crate::Plugin;
use crate::session::{Error, Session};

use super::plugin::PayloadStrategy;

type Inventory = BTreeMap<&'static str, Box<dyn Plugin>>;

macro_rules! register_plugin {
    ($($name:literal => $instance:expr),+) => {
        pub(super) fn register(registrar: &mut impl $crate::plugins::manager::PluginRegistrar) {
            $(
                registrar.register($name, $instance);
            )*
        }
    };
}

pub(crate) use register_plugin;

pub(crate) trait PluginRegistrar {
    fn register<P: Plugin + 'static>(&mut self, name: &'static str, plugin: P);
}

impl PluginRegistrar for Inventory {
    #[inline]
    fn register<P: Plugin + 'static>(&mut self, name: &'static str, plugin: P) {
        self.insert(name, Box::new(plugin));
    }
}

pub(crate) static INVENTORY: LazyLock<Mutex<Inventory>> = LazyLock::new(|| {
    let mut ps = Inventory::new();
    super::add_defaults(&mut ps);
    Mutex::new(ps)
});

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

pub(crate) async fn setup(options: &Options) -> Result<&'static mut dyn Plugin, Error> {
    let Some(plugin_name) = options.plugin.as_ref() else {
        return Err("no plugin selected".to_owned());
    };
    let Some(plugin) = INVENTORY
        .lock()
        .unwrap()
        .remove(plugin_name.as_str())
        .map(Box::leak)
    else {
        return Err(format!(
            "{} is not a valid plugin name, run with --list-plugins to see the list of available plugins",
            plugin_name
        ));
    };

    plugin.setup(options).await?;

    Ok(plugin)
}

pub(crate) async fn run(
    plugin: &'static mut dyn Plugin,
    session: Arc<Session>,
) -> Result<(), Error> {
    let single = matches!(plugin.payload_strategy(), PayloadStrategy::Single);
    let override_payload = plugin.override_payload();
    let combinations = session.combinations(override_payload, single)?;
    let unreachables: Arc<DashSet<Arc<str>>> = Arc::new(DashSet::default());

    // spawn worker tasks
    for _ in 0..session.options.concurrency {
        task::spawn(worker(plugin, unreachables.clone(), session.clone()));
    }

    if !session.options.quiet {
        // start statistics reporting
        let stat_sess = session.clone();
        tokio::task::spawn(async move {
            stat_sess.report_runtime_statistics().await;
        });
    }

    // loop credentials for this session
    for creds in combinations {
        // exit on ctrl-c if we have to, otherwise send the new credentials to the workers
        if session.is_stop() {
            log::debug!("exiting loop");
            return Ok(());
        } else if let Err(e) = session.send_credentials(creds).await {
            log::error!("{}", e);
        }
    }

    Ok(())
}

async fn worker(plugin: &dyn Plugin, unreachables: Arc<DashSet<Arc<str>>>, session: Arc<Session>) {
    log::debug!("worker started");

    let timeout = time::Duration::from_millis(session.options.timeout);
    let retry_time: time::Duration = time::Duration::from_millis(session.options.retry_time);

    while let Ok(creds) = session.recv_credentials().await {
        if session.is_stop() {
            log::debug!("exiting worker");
            break;
        }

        let mut errors = 0;
        let mut attempt = 0;

        while attempt < session.options.retries && !session.is_stop() {
            // perform random jitter if needed
            if session.options.jitter_max > 0 {
                let ms = rand::rng()
                    .random_range(session.options.jitter_min..=session.options.jitter_max);
                if ms > 0 {
                    log::debug!("jitter of {} ms", ms);
                    tokio::time::sleep(time::Duration::from_millis(ms)).await;
                }
            }

            attempt += 1;

            // skip attempt if we had enough failures from this specific target
            if !unreachables.contains(creds.target.as_str()) {
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
                            tokio::time::sleep(retry_time).await;
                            continue;
                        } else {
                            // add this target to the list of unreachable in order to avoi
                            // pointless attempts
                            unreachables.insert(Arc::from(creds.target.as_str()));

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
                        if let Some(loots) = loot {
                            for loot in loots {
                                session.add_loot(loot).await.unwrap();
                            }
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
