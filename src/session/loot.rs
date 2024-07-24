use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{fmt, path::Path};

use ansi_term::Colour;
use chrono::{DateTime, Local};
use clap::ValueEnum;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::session::Error;

#[derive(ValueEnum, Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) enum OutputFormat {
    #[default]
    Text,
    CSV,
    JSONL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Loot {
    found_at: DateTime<Local>,
    target: String,
    plugin: String,
    data: IndexMap<String, String>,
    partial: bool,
}

impl Loot {
    pub fn new<I: IntoIterator<Item = (String, String)>>(
        plugin: &str,
        target: &str,
        iterable: I,
    ) -> Self {
        let found_at = chrono::Local::now();
        let target = target.to_string();
        let plugin = plugin.to_string();
        let data = IndexMap::from_iter(iterable);
        let partial = false;
        Self {
            found_at,
            target,
            plugin,
            data,
            partial,
        }
    }

    pub fn get_target(&self) -> &str {
        &self.target
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    pub fn set_partial(mut self) -> Self {
        self.partial = true;
        self
    }

    fn found_at_string(&self) -> String {
        self.found_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    fn to_text(&self) -> Result<String, Error> {
        let data = self
            .data
            .keys()
            .map(|k| format!("{}={}", k, self.data.get(k).unwrap()))
            .collect::<Vec<String>>()
            .join("\t");

        Ok(if self.target.is_empty() {
            format!("[{}] ({}) {}", self.found_at_string(), &self.plugin, data)
        } else {
            format!(
                "[{}] ({}) <{}> {}",
                self.found_at_string(),
                &self.plugin,
                &self.target,
                data
            )
        })
    }

    fn to_csv(&self, path: &str) -> Result<String, Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        if !Path::new(path).exists() {
            wtr.write_record(["found_at", "plugin", "target", "data"])
                .map_err(|e| e.to_string())?;
        }

        let data = self
            .data
            .keys()
            .map(|k| format!("{}={}", k, self.data.get(k).unwrap()))
            .collect::<Vec<String>>()
            .join(";");

        wtr.write_record([&self.found_at_string(), &self.plugin, &self.target, &data])
            .map_err(|e| e.to_string())?;

        String::from_utf8(wtr.into_inner().unwrap()).map_err(|e| e.to_string())
    }

    pub fn append_to_file(&self, path: &str, format: &OutputFormat) -> Result<(), Error> {
        let data = match format {
            OutputFormat::JSONL => self.to_json()?,
            OutputFormat::Text => self.to_text()?,
            OutputFormat::CSV => self.to_csv(path)?,
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{}", data.trim()).map_err(|e| e.to_string())
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

        if self.target.is_empty() {
            write!(
                f,
                "[{}] ({}) {}",
                self.found_at_string(),
                &self.plugin,
                str.trim_end()
            )
        } else {
            write!(
                f,
                "[{}] ({}) <{}> {}",
                self.found_at_string(),
                &self.plugin,
                &self.target,
                str.trim_end()
            )
        }
    }
}
