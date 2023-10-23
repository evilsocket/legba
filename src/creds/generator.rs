use std::fs::File;
use std::io::{prelude::*, BufReader, Lines};

use crate::creds::expression::Expression;
use crate::creds::permutator::Permutator;
use crate::session::Error;

#[derive(Default, Debug)]
pub(crate) struct Generator {
    constant: Option<String>,
    lines: Option<Lines<BufReader<File>>>,
    permutator: Option<Permutator>,
    current: usize,
    elements: usize,
}

impl Generator {
    fn from_constant_value(value: String) -> Self {
        Generator {
            constant: Some(value),
            permutator: None,
            elements: 1,
            current: 0,
            lines: None,
        }
    }

    fn from_wordlist(path: String) -> Result<Self, Error> {
        log::debug!("loading wordlist from {} ...", &path);

        // count the number of lines first
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let elements = reader.lines().count();

        // create actual reader
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        Ok(Generator {
            constant: None,
            permutator: None,
            elements,
            current: 0,
            lines: Some(reader.lines()),
        })
    }

    fn from_range(charset: String, min_length: usize, max_length: usize) -> Result<Self, Error> {
        if min_length == 0 {
            return Err("min length can't be zero".to_owned());
        } else if min_length > max_length {
            return Err("min length can't be greater than max length".to_owned());
        }

        let gen = Permutator::new(charset.chars().collect(), min_length, max_length);
        let tot = gen.search_space_size();
        let generator = Some(gen);

        Ok(Generator {
            lines: None,
            permutator: generator,
            current: 0,
            elements: tot,
            constant: None,
        })
    }

    pub fn new(expr: Expression) -> Result<Self, Error> {
        match expr {
            Expression::Constant { value } => Ok(Self::from_constant_value(value)),
            Expression::Wordlist { filename } => Self::from_wordlist(filename),
            Expression::Range { min, max, charset } => Self::from_range(charset, min, max),
        }
    }

    pub fn search_space_size(&self) -> usize {
        self.elements
    }

    fn next_line(&mut self) -> Option<String> {
        if let Some(lines) = &mut self.lines {
            if let Some(res) = lines.next() {
                if let Ok(line) = res {
                    return Some(line);
                } else {
                    log::error!("could not read line: {:?}", res.err());
                }
            }
        }
        None
    }
}

impl Iterator for Generator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.elements {
            self.current += 1;
            if let Some(value) = &self.constant {
                return Some(value.to_owned());
            } else if self.lines.is_some() {
                return self.next_line();
            } else if let Some(permutator) = self.permutator.as_mut() {
                return permutator.next();
            }
        }
        None
    }
}
