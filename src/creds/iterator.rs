use std::fs::File;
use std::io::{prelude::*, BufReader, Lines};

use crate::creds::expression::Expression;
use crate::creds::permutator::Permutator;
use crate::session::Error;

pub(crate) trait Iterator: std::iter::Iterator<Item = String> {
    fn search_space_size(&self) -> usize;
}

pub(crate) fn new(expr: Expression) -> Result<Box<dyn Iterator>, Error> {
    match expr {
        Expression::Constant { value } => {
            let gen = Constant::new(value)?;
            Ok(Box::new(gen))
        }
        Expression::Wordlist { filename } => {
            let gen = Wordlist::new(filename)?;
            Ok(Box::new(gen))
        }
        Expression::Range { min, max, charset } => {
            let gen = Range::new(charset, min, max)?;
            Ok(Box::new(gen))
        }
        Expression::Glob { pattern } => {
            let gen = Glob::new(pattern)?;
            Ok(Box::new(gen))
        }
    }
}

pub(crate) fn empty() -> Result<Box<dyn Iterator>, Error> {
    Ok(Box::new(Empty::new()))
}

struct Empty {}

impl Empty {
    pub fn new() -> Self {
        Self {}
    }
}

impl Iterator for Empty {
    fn search_space_size(&self) -> usize {
        0
    }
}

impl std::iter::Iterator for Empty {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

struct Constant {
    done: bool,
    value: String,
}

impl Constant {
    pub fn new(value: String) -> Result<Self, Error> {
        let done = false;
        Ok(Self { done, value })
    }
}

impl Iterator for Constant {
    fn search_space_size(&self) -> usize {
        1
    }
}

impl std::iter::Iterator for Constant {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = true;
            Some(self.value.to_owned())
        }
    }
}

struct Wordlist {
    lines: Lines<BufReader<File>>,
    current: usize,
    elements: usize,
}

impl Wordlist {
    pub fn new(path: String) -> Result<Self, Error> {
        log::debug!("loading wordlist from {} ...", &path);

        // count the number of lines first
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let elements = reader.lines().count();

        // create actual reader
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        Ok(Self {
            elements,
            current: 0,
            lines: reader.lines(),
        })
    }
}

impl Iterator for Wordlist {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl std::iter::Iterator for Wordlist {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.elements {
            self.current += 1;
            if let Some(res) = self.lines.next() {
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

struct Range {
    permutator: Permutator,
    elements: usize,
}

impl Range {
    pub fn new(charset: String, min_length: usize, max_length: usize) -> Result<Self, Error> {
        if min_length == 0 {
            return Err("min length can't be zero".to_owned());
        } else if min_length > max_length {
            return Err("min length can't be greater than max length".to_owned());
        }

        let permutator = Permutator::new(charset.chars().collect(), min_length, max_length);
        let elements = permutator.search_space_size();

        Ok(Self {
            permutator,
            elements,
        })
    }
}

impl Iterator for Range {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl std::iter::Iterator for Range {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.permutator.next()
    }
}

struct Glob {
    paths: glob::Paths,
    elements: usize,
}

impl Glob {
    pub fn new(pattern: String) -> Result<Self, Error> {
        // validate the pattern and count the elements first
        let paths = match glob::glob(&pattern) {
            Err(e) => return Err(e.to_string()),
            Ok(paths) => paths,
        };
        let elements = paths.count();
        let paths = glob::glob(&pattern).unwrap();

        Ok(Self { paths, elements })
    }
}

impl Iterator for Glob {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl std::iter::Iterator for Glob {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.paths.next() {
            if let Ok(path) = next {
                return Some(path.to_str().unwrap().to_owned());
            } else {
                log::error!("glob error: {:?}", next);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::creds::{iterator, Expression};

    #[test]
    fn can_handle_constant() {
        let gen = iterator::new(Expression::Constant {
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

        let gen = iterator::new(Expression::Wordlist {
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
        let gen = iterator::new(Expression::Range {
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

        let gen = iterator::new(Expression::Glob {
            pattern: format!("{}/*.txt", tmpdirname),
        })
        .unwrap();
        let tot = gen.search_space_size();
        let vec: Vec<String> = gen.collect();

        assert_eq!(tot, expected.len());
        assert_eq!(vec, expected);
    }
}
