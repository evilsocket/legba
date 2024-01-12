use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::session::Error;

lazy_static! {
    static ref ARG_VALUE_PARSER: Regex =
        Regex::new(r"(?m)\{\s*\$(\w+)(\s+or\s+([^}]+))?\}").unwrap();
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub(crate) struct Recipe {
    pub description: String,
    pub author: String,
    pub plugin: String,
    pub args: HashMap<String, String>,
}

impl Recipe {
    pub fn from_path(path: &str) -> Result<Self, Error> {
        let yaml = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let recipe: Self = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

        Ok(recipe)
    }

    fn parse_context(context: &str) -> Result<HashMap<String, String>, Error> {
        let mut ctx = HashMap::new();

        for pair in context.split('&') {
            if pair.contains('=') {
                let (key, value) = pair.split_once('=').unwrap();
                ctx.insert(key.to_owned(), value.to_owned());
            }
        }

        Ok(ctx)
    }

    fn parse_arg_value(value: &str, ctx: &HashMap<String, String>) -> Result<String, Error> {
        let mut parsed = value.to_owned();

        for cap in ARG_VALUE_PARSER.captures_iter(value) {
            let expr = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str();
            let var_value = if let Some(val) = ctx.get(var_name) {
                // get variable from context
                val
            } else if let Some(def) = cap.get(3) {
                // get variable from default
                def.as_str()
            } else {
                return Err(format!("no '{}' specified for recipe", var_name));
            };

            parsed = parsed.replace(expr, var_value);
        }

        Ok(parsed)
    }

    pub fn to_argv(&self, context: &str) -> Result<Vec<String>, Error> {
        let mut argv = vec![
            "".to_owned(), // simulates argv[0]
            self.plugin.to_owned(),
        ];

        let ctx = Self::parse_context(context)?;
        for (arg_name, arg_value) in &self.args {
            argv.push(format!("--{}", arg_name));
            argv.push(Self::parse_arg_value(arg_value, &ctx)?);
        }

        Ok(argv)
    }
}
