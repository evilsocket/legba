use crate::creds;

pub(crate) struct Empty {}

impl Empty {
    pub fn new() -> Self {
        Self {}
    }
}

impl creds::Iterator for Empty {
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
