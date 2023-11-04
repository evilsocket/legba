use crate::{creds, session::Error};

pub(crate) struct Constant {
    done: bool,
    value: String,
}

impl Constant {
    pub fn new(value: String) -> Result<Self, Error> {
        let done = false;
        Ok(Self { done, value })
    }
}

impl creds::Iterator for Constant {
    fn search_space_size(&self) -> usize {
        1
    }
}

impl creds::IteratorClone for Constant {
    fn create_boxed_copy(&self) -> Box<dyn creds::Iterator> {
        Box::new(Self::new(self.value.clone()).unwrap())
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

#[cfg(test)]
mod tests {
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
}
