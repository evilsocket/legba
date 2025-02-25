mod combinator;
mod expression;
mod iterator;

pub(crate) use combinator::{Combinator, IterationStrategy};
pub(crate) use expression::{Expression, parse_expression};
pub(crate) use iterator::{Iterator, IteratorClone};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Default, Clone, Debug)]
pub(crate) struct Credentials {
    pub target: String,
    pub username: String,
    pub password: String,
}

impl Credentials {
    #[inline(always)]
    pub fn single(&self) -> &str {
        if self.username.is_empty() {
            &self.password
        } else {
            &self.username
        }
    }
}
