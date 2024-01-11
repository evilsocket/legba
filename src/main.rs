use std::env;
use std::io;
use std::time;

use clap::{CommandFactory, Parser};
use creds::Credentials;

#[cfg(not(windows))]
use rlimit::{setrlimit, Resource};

mod creds;
mod options;
mod plugins;
mod report;
mod session;
mod utils;

pub(crate) use crate::options::Options;
pub(crate) use crate::plugins::Plugin;
pub(crate) use crate::session::Session;

fn setup() -> Result<Options, session::Error> {
    if env::var_os("RUST_LOG").is_none() {
        // set `RUST_LOG=debug` to see debug logs
        env::set_var("RUST_LOG", "info,blocking=off,pavao=off,fast_socks5=off");
    }

    env_logger::builder()
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let options: Options = Options::parse();

    if let Some(shell) = options.generate_completions {
        clap_complete::generate(shell, &mut Options::command(), "legba", &mut io::stdout());
        std::process::exit(0);
    }

    // list plugins and exit
    if options.list_plugins {
        plugins::manager::list();
        std::process::exit(0);
    }

    // set file descriptors limits
    #[cfg(not(windows))]
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
