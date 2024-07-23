use crate::{creds, session::Error};

pub(crate) struct Multi {
    curr_it: usize,
    num_its: usize,
    elements: usize,
    iters: Vec<Box<dyn creds::Iterator<Item = String>>>,
}

impl Multi {
    pub fn new(iters: Vec<Box<dyn creds::Iterator<Item = String>>>) -> Result<Self, Error> {
        log::debug!("loading Multi with {} iterators ...", iters.len());

        // count the number of items for each iterator
        let curr_it = 0;
        let num_its = iters.len();
        let mut elements = 0;
        for it in iters.iter() {
            elements += it.search_space_size();
        }

        Ok(Self {
            elements,
            curr_it,
            num_its,
            iters,
        })
    }
}

impl creds::Iterator for Multi {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl creds::IteratorClone for Multi {
    fn create_boxed_copy(&self) -> Box<dyn creds::Iterator> {
        Box::new(Self::new(self.iters.clone()).unwrap())
    }
}

impl std::iter::Iterator for Multi {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_it < self.num_its {
            let curr = &mut self.iters[self.curr_it];
            let next = curr.next();
            return if next.is_none() {
                self.curr_it += 1;
                self.next()
            } else {
                next
            };
        }
        None
    }
}
