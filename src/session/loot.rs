use std::fmt;
use std::fs::OpenOptions;
use std::io::prelude::*;

use ansi_term::Colour;
use clap::ValueEnum;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::session::Error;

#[derive(ValueEnum, Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) enum OutputFormat {
    #[default]
    Text,
    JSONL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Loot {
    data: IndexMap<String, String>,
    pub partial: bool,
}

impl Loot {
    pub fn from<I: IntoIterator<Item = (String, String)>>(iterable: I) -> Self {
        Self {
            data: IndexMap::from_iter(iterable),
            partial: false,
        }
    }

    pub fn set_partial(mut self) -> Self {
        self.partial = true;
        self
    }

    pub fn append_to_file(&self, path: &str, format: &OutputFormat) -> Result<(), Error> {
        let data = match format {
            OutputFormat::JSONL => serde_json::to_string(self).map_err(|e| e.to_string())?,
            OutputFormat::Text => self
                .data
                .keys()
                .map(|k| format!("{}={}", k, self.data.get(k).unwrap()))
                .collect::<Vec<String>>()
                .join("\t"),
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{}", data).map_err(|e| e.to_string())
    }
}

impl fmt::Display for Loot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        for (key, value) in &self.data {
            if !value.is_empty() {
                str.push_str(&format!("{}={} ", key, Colour::Green.bold().paint(value)));
            }
        }
        write!(f, "{}", str.trim_end())
    }
}
