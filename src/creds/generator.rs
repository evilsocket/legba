use std::fs::File;
use std::io::{prelude::*, BufReader, Lines};

use glob;

use crate::creds::expression::Expression;
use crate::creds::permutator::Permutator;
use crate::session::Error;

#[derive(Default, Debug)]
pub(crate) struct Generator {
    constant: Option<String>,
    lines: Option<Lines<BufReader<File>>>,
    permutator: Option<Permutator>,
    paths: Option<glob::Paths>,
    current: usize,
    elements: usize,
}

// TODO: this should probably be refactored into a trait
impl Generator {
    fn from_constant_value(value: String) -> Self {
        Generator {
            constant: Some(value),
            permutator: None,
            paths: None,
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
            paths: None,
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
            paths: None,
            current: 0,
            elements: tot,
            constant: None,
        })
    }

    fn from_glob(pattern: String) -> Result<Self, Error> {
        // validate the pattern and count the elements first
        let paths = match glob::glob(&pattern) {
            Err(e) => return Err(e.to_string()),
            Ok(paths) => paths,
        };

        Ok(Generator {
            constant: None,
            lines: None,
            permutator: None,
            paths: Some(glob::glob(&pattern).unwrap()),
            current: 0,
            elements: paths.count(),
        })
    }

    pub fn new(expr: Expression) -> Result<Self, Error> {
        match expr {
            Expression::Constant { value } => Ok(Self::from_constant_value(value)),
            Expression::Wordlist { filename } => Self::from_wordlist(filename),
            Expression::Range { min, max, charset } => Self::from_range(charset, min, max),
            Expression::Glob { pattern } => Self::from_glob(pattern),
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
            } else if let Some(paths) = &mut self.paths {
                if let Some(next) = paths.next() {
                    if let Ok(path) = next {
                        return Some(path.to_str().unwrap().to_owned());
                    } else {
                        log::error!("glob error: {:?}", next);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::Generator;
    use crate::creds::Expression;

    #[test]
    fn can_handle_constant() {
        let gen = Generator::new(Expression::Constant {
            value: "hi".to_owned(),
        })
        .unwrap();
        let tot = gen.search_space_size();
        let vec: Vec<String> = gen.collect();

        assert_eq!(tot, 1);
        assert_eq!(vec, vec!["hi".to_owned()]);
    }

    #[test]
    fn can_handle_wordlist() {
        let num_items = 3;
        let mut expected = vec![];
        let tmpdir = tempfile::tempdir().unwrap();
        let tmppath = tmpdir.path().join("wordlist.txt");
        let mut tmpwordlist = File::create(&tmppath).unwrap();

        for i in 0..num_items {
            write!(tmpwordlist, "item{}\n", i).unwrap();
            expected.push(format!("item{}", i));
        }
        tmpwordlist.flush().unwrap();
        drop(tmpwordlist);

        let gen = Generator::new(Expression::Wordlist {
            filename: tmppath.to_str().unwrap().to_owned(),
        })
        .unwrap();
        let tot = gen.search_space_size();
        let vec: Vec<String> = gen.collect();

        assert_eq!(tot, num_items);
        assert_eq!(vec, expected);
    }

    #[test]
    fn can_handle_range() {
        let expected = vec![
            "a", "b", "c", "aa", "ab", "ac", "ba", "bb", "bc", "ca", "cb", "cc",
        ];
        let gen = Generator::new(Expression::Range {
            min: 1,
            max: 2,
            charset: "abc".to_owned(),
        })
        .unwrap();
        let tot = gen.search_space_size();
        let vec: Vec<String> = gen.collect();

        assert_eq!(tot, expected.len());
        assert_eq!(vec, expected);
    }

    #[test]
    fn can_handle_glob() {
        let num_files = 3;
        let tmpdir = tempfile::tempdir().unwrap();
        let tmpdirname = tmpdir.path().to_str().unwrap().to_owned();
        let mut expected = vec![];

        for i in 0..num_files {
            let filename = format!("test{}.txt", i);
            let tmppath = tmpdir.path().join(&filename);
            let mut tmpfile = File::create(&tmppath).unwrap();

            write!(tmpfile, "test\n").unwrap();
            tmpfile.flush().unwrap();
            drop(tmpfile);

            expected.push(format!("{}/{}", &tmpdirname, filename));
        }

        let gen = Generator::new(Expression::Glob {
            pattern: format!("{}/*.txt", tmpdirname),
        })
        .unwrap();
        let tot = gen.search_space_size();
        let vec: Vec<String> = gen.collect();

        assert_eq!(tot, expected.len());
        assert_eq!(vec, expected);
    }
}
