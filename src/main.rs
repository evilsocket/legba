use std::env;
use std::io;
use std::time;

use clap::parser::ValueSource;
use clap::ArgMatches;
use clap::{CommandFactory, Parser};
use creds::Credentials;

use env_logger::Target;
#[cfg(not(windows))]
use rlimit::{Resource, setrlimit};
use serde_json::Value;

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

fn update_field_by_name(options: &mut Options, field_name: &str, matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize current options to JSON
    let mut options_value: Value = serde_json::to_value(&options)?;

    // Get the object map
    if let Value::Object(ref mut map) = options_value {
        // Get the current value to determine its type
        if let Some(current_value) = map.get(field_name) {
            match current_value {
                Value::Null | Value::String(_) => {
                    if let Some(val) = matches.get_one::<String>(field_name) {
                        log::debug!("updating field: {:?} = {:?}", &field_name, &val);
                        map.insert(field_name.to_string(), Value::String(val.clone()));
                    } else {
                        return Err(format!("Missing value for field: {:?}", &field_name).into());
                    }
                }
                Value::Number(_) => {
                    if let Some(val) = matches.get_one::<usize>(field_name) {
                        log::debug!("updating field: {:?} = {:?}", &field_name, &val);
                        map.insert(field_name.to_string(), Value::Number((*val).into()));
                    } else {
                        return Err(format!("Missing value for field: {:?}", &field_name).into());
                    }
                }
                Value::Bool(_) => {
                    if let Some(val) = matches.get_one::<bool>(field_name) {
                        log::debug!("updating field: {:?} = {:?}", &field_name, &val);
                        map.insert(field_name.to_string(), Value::Bool(*val));
                    } else {
                        return Err(format!("Missing value for field: {:?}", &field_name).into());
                    }
                }
                _ => return Err(format!("Unknown or unsupported type for field: {:?} (current_value={:?})", &field_name, &current_value).into())
            }
        }
    
    } else {
        return Err(format!("Invalid options structure: expected JSON object for field: {:?}", &field_name).into());
    }

    // Deserialize back to Options
    *options = serde_json::from_value(options_value)?;

    Ok(())
}

fn update_options_selectively(options: &mut Options, argv: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Options::command();
    let matches = cmd.clone().try_get_matches_from(argv)?;
    
    // Update only the fields that were explicitly provided
    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if matches.contains_id(id) && 
           matches.value_source(id) == Some(ValueSource::CommandLine) && id != "plugin" && id != "P" && id != "R" && id != "recipe" {
            update_field_by_name(options, id, &matches)?;
        }
    }
    
    Ok(())
}

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

        let empty = "".to_string();
        let context = options.plugin.as_ref().unwrap_or(&empty);
        log::debug!("  context={:?}", &context);
        let argv = recipe.to_argv(&context)?;
        // this would reset options not set in the recipe to their default values
        // even if specified in the command line, see https://github.com/evilsocket/legba/issues/66
        // options.try_update_from(argv).map_err(|e| e.to_string())?;
        // create new options from this argv
        let mut new_options = Options::parse_from(&argv);

        // get original argv from std::env::args_os()
        let original_argv: Vec<String> = std::env::args_os()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();
        log::debug!("original argv: {:?}", &original_argv);
        // allow whatever the user specified in the command line to override 
        // the values from the recipe, see https://github.com/evilsocket/legba/issues/66
        update_options_selectively(&mut new_options, &original_argv).map_err(|e| e.to_string())?;

        options = new_options;
    }

    log::debug!("{}", serde_json::to_string_pretty(&options).unwrap());

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
