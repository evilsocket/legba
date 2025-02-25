use crate::{creds, session::Error};

pub(crate) struct Glob {
    pattern: String,
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

        Ok(Self {
            pattern,
            paths,
            elements,
        })
    }
}

impl creds::Iterator for Glob {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl creds::IteratorClone for Glob {
    fn create_boxed_copy(&self) -> Box<dyn creds::Iterator> {
        Box::new(Self {
            pattern: self.pattern.to_owned(),
            elements: self.elements,
            paths: glob::glob(&self.pattern).unwrap(),
        })
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

    use crate::creds::{Expression, iterator};

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

            writeln!(tmpfile, "test").unwrap();
            tmpfile.flush().unwrap();
            drop(tmpfile);

            expected.push(format!("{}/{}", &tmpdirname, filename));
        }

        let iter = iterator::new(Expression::Glob {
            pattern: format!("{}/*.txt", tmpdirname),
        })
        .unwrap();
        let tot = iter.search_space_size();
        let vec: Vec<String> = iter.collect();

        assert_eq!(tot, expected.len());
        assert_eq!(vec, expected);
    }
}
