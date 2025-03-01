use std::{collections::HashMap, path::PathBuf};

use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::session::Error;

use self::context::Context;

mod context;
mod interactive;

const ARG_EXPRESSION_ERROR: &str =
    "argument expression must be in the form of {$name} or {$name or default_value}";

const RESERVED_VAR_MAMES: [&str; 3] = ["username", "password", "payload"];

static ARG_VALUE_PARSER: Lazy<Regex> = lazy_regex!(r"(?m)\{\s*\$([\w\.]+)(\s+or\s+([^}]+))?\}");

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub(crate) struct Recipe {
    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub interactive: bool,
    pub description: String,
    pub author: String,
    pub plugin: String,
    pub args: HashMap<String, String>,
}

impl Recipe {
    pub fn from_path(path: &str) -> Result<Self, Error> {
        let path = PathBuf::from(path);
        log::debug!(
            "loading recipe from {:?} (is_dir={:?})",
            &path,
            path.is_dir()
        );
        let mut path = if path.is_dir() {
            path.join("recipe.yml")
        } else {
            path
        };

        let yaml = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut recipe: Self = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

        // remove filename portion
        path.pop();
        // set recipe path
        recipe.path = std::fs::canonicalize(&path)
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();

        Ok(recipe)
    }

    fn parse_arg(&self, expr: &str, ctx: &mut Context) -> Result<String, Error> {
        let mut parsed = expr.to_owned();

        for cap in ARG_VALUE_PARSER.captures_iter(expr) {
            let expr: &str = cap.get(0).ok_or(ARG_EXPRESSION_ERROR)?.as_str();
            let var_name = cap.get(1).ok_or(ARG_EXPRESSION_ERROR)?.as_str();
            let var_default = cap.get(3).map(|m| m.as_str());

            let var_value = if RESERVED_VAR_MAMES.contains(&var_name) {
                // if reserved variable, leave it as it is for further processing down the line
                continue;
            } else if let Some(val) = ctx.get(var_name) {
                // get variable from context
                val.to_owned()
            } else if self.interactive {
                // get from user if interactive
                interactive::read_arg_from_user(var_name, var_default)?
            } else if let Some(def) = var_default {
                // get variable from default if provided
                def.to_owned()
            } else {
                return Err(format!("no '{}' variable specified for recipe", var_name));
            };

            // cache value in context
            ctx.add(var_name, &var_value);

            parsed = parsed.replace(expr, &var_value);
        }

        Ok(parsed)
    }

    pub fn to_argv(&self, context: &str) -> Result<Vec<String>, Error> {
        let mut argv = vec![
            "".to_owned(), // simulates argv[0]
            self.plugin.to_owned(),
        ];

        let mut ctx = Context::parse(context)?;

        // add default variables
        ctx.add("recipe.path", &self.path);

        for (arg_name, arg_value) in &self.args {
            argv.push(format!("--{}", arg_name));
            if arg_value != "null" {
                argv.push(self.parse_arg(arg_value, &mut ctx)?);
            }
        }

        // print context
        for (key, val) in ctx.iter() {
            log::info!("  {}={}", key, val);
        }

        Ok(argv)
    }
}
