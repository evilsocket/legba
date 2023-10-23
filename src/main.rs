use std::env;
use std::time;

use clap::Parser;
use creds::Credentials;
use rlimit::{setrlimit, Resource};
use tokio::task;

mod creds;
mod options;
mod plugins;
mod report;
mod session;
mod utils;

pub(crate) use crate::options::Options;
pub(crate) use crate::plugins::Plugin;
pub(crate) use crate::session::Session;

// TODO: Plugin specific documentation.
// TODO: Benchmark table.

fn setup() -> Result<Options, session::Error> {
    print!(
        "{} v{}\n\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    if env::var_os("RUST_LOG").is_none() {
        // set `RUST_LOG=debug` to see debug logs
        env::set_var("RUST_LOG", "info,blocking=off");
    }

    env_logger::builder()
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let options: Options = Options::parse();

    // list plugins and exit
    if options.list_plugins {
        plugins::manager::list();
        std::process::exit(0);
    }

    // set file descriptors limits
    setrlimit(Resource::NOFILE, options.ulimit, options.ulimit).map_err(|e| {
        format!(
            "can't adjust max open files limit to {}: {:?}",
            options.ulimit, e
        )
    })?;

    Ok(options)
}

#[tokio::main]
async fn main() -> Result<(), session::Error> {
    // initialize and parse command line
    let opts = setup()?;

    // create the session object with runtime information
    // NOTE: from this moment on we use session.options
    let session = Session::new(opts.clone())?;

    // get selected plugin and configure it
    let plugin = plugins::manager::setup(&session.options).map_err(|e| {
        // set stop signal if the plugin failed to load
        session.set_stop();
        e
    })?;

    if !session.options.quiet {
        // start statistics reporting
        task::spawn(report::statistics(session.clone()));
    }

    let start = time::Instant::now();

    // start plugin
    plugins::manager::run(plugin, session.clone()).await?;

    let one_sec = time::Duration::from_secs(1);
    while !session.is_finished() {
        std::thread::sleep(one_sec);
    }

    log::info!("runtime {:?}", start.elapsed());

    // sometimes the program hangs waiting for some remaining tokio tasks
    // to complete - we just exit(0) to avoid this.
    std::process::exit(0);

    #[allow(unreachable_code)]
    Ok(())
}
