use std::env;
use std::io;
use std::time;

use clap::{CommandFactory, Parser};
use creds::Credentials;

use env_logger::Target;
#[cfg(not(windows))]
use rlimit::{Resource, setrlimit};

mod api;
mod creds;
mod options;
mod plugins;
mod recipe;
mod report;
mod session;
mod utils;

pub(crate) use crate::options::Options;
pub(crate) use crate::plugins::Plugin;
use crate::recipe::Recipe;
pub(crate) use crate::session::Session;

fn setup() -> Result<Options, session::Error> {
    if env::var_os("RUST_LOG").is_none() {
        // set `RUST_LOG=debug` to see debug logs
        unsafe {
            env::set_var(
                "RUST_LOG",
                "info,blocking=off,pavao=off,fast_socks5=off,actix_server=warn",
            );
        }
    }

    env_logger::builder()
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        .target(Target::Stdout)
        .init();

    let mut options: Options = Options::parse();

    // generate shell completions and exit
    if let Some(shell) = options.generate_completions {
        clap_complete::generate(shell, &mut Options::command(), "legba", &mut io::stdout());
        std::process::exit(0);
    }

    // list plugins and exit
    if options.list_plugins {
        plugins::manager::list();
        std::process::exit(0);
    }

    print!(
        "{} v{}\n\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // load recipe
    if let Some(recipe_path) = options.recipe.as_ref() {
        let recipe = Recipe::from_path(recipe_path)?;

        log::info!("recipe: {} ({})", recipe_path, recipe.description);

        // get new argv from recipe
        let argv = recipe.to_argv(options.plugin.as_ref().unwrap_or(&"".to_string()))?;

        log::debug!("  argv={:?}", &argv);

        // repopulate the options from this argv
        options.try_update_from(argv).map_err(|e| e.to_string())?;
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

async fn start_session(opts: Options) -> Result<(), session::Error> {
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
        tokio::time::sleep(one_sec).await;
    }

    log::info!("runtime {:?}", start.elapsed());

    // sometimes the program hangs waiting for some remaining tokio tasks
    // to complete - we just exit(0) to avoid this.
    std::process::exit(0);

    #[allow(unreachable_code)]
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), session::Error> {
    // initialize and parse command line
    let opts = setup()?;
    if opts.api.is_some() {
        // start api
        api::start(opts).await
    } else {
        // start cli session
        start_session(opts).await
    }
}
