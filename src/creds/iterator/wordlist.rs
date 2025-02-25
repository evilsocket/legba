use std::{
    fs::File,
    io::{BufReader, Lines, prelude::*},
};

use crate::{creds, session::Error};

pub(crate) struct Wordlist {
    path: String,
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
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        Ok(Self {
            path,
            elements,
            current: 0,
            lines: reader.lines(),
        })
    }
}

impl creds::Iterator for Wordlist {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl creds::IteratorClone for Wordlist {
    fn create_boxed_copy(&self) -> Box<dyn creds::Iterator> {
        Box::new(Self::new(self.path.clone()).unwrap())
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::creds::{Expression, iterator};

    #[test]
    fn can_handle_wordlist() {
        let num_items = 3;
        let mut expected = vec![];
        let tmpdir = tempfile::tempdir().unwrap();
        let tmppath = tmpdir.path().join("wordlist.txt");
        let mut tmpwordlist = File::create(&tmppath).unwrap();

        for i in 0..num_items {
            writeln!(tmpwordlist, "item{}", i).unwrap();
            expected.push(format!("item{}", i));
        }
        tmpwordlist.flush().unwrap();
        drop(tmpwordlist);

        let iter = iterator::new(Expression::Wordlist {
            filename: tmppath.to_str().unwrap().to_owned(),
        })
        .unwrap();
        let tot = iter.search_space_size();
        let vec: Vec<String> = iter.collect();

        assert_eq!(tot, num_items);
        assert_eq!(vec, expected);
    }
}
