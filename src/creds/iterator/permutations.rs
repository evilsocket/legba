use crate::{creds, session::Error};

use super::permutator::Permutator;

pub(crate) struct Permutations {
    permutator: Permutator,
    elements: usize,
}

impl Permutations {
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

impl creds::Iterator for Permutations {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl std::iter::Iterator for Permutations {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.permutator.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::creds::{iterator, Expression};

    #[test]
    fn can_handle_permutations() {
        let expected = vec![
            "a", "b", "c", "aa", "ab", "ac", "ba", "bb", "bc", "ca", "cb", "cc",
        ];
        let gen = iterator::new(Expression::Permutations {
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
}
