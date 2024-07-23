use crate::creds::expression::Expression;
use crate::session::Error;

mod constant;
mod glob;
mod multi;
mod permutations;
mod permutator;
mod range;
mod wordlist;

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub(crate) trait Iterator: IteratorClone + std::iter::Iterator<Item = String> {
    fn search_space_size(&self) -> usize;
}

pub(crate) trait IteratorClone {
    fn create_boxed_copy(&self) -> Box<dyn Iterator>;
}

impl Clone for Box<dyn Iterator> {
    fn clone(&self) -> Self {
        self.create_boxed_copy()
    }
}

pub(crate) fn new(expr: Expression) -> Result<Box<dyn Iterator>, Error> {
    match expr {
        Expression::Constant { value } => {
            let it = constant::Constant::new(value)?;
            Ok(Box::new(it))
        }
        Expression::Wordlist { filename } => {
            let it = wordlist::Wordlist::new(filename)?;
            Ok(Box::new(it))
        }
        Expression::Permutations { min, max, charset } => {
            let it = permutations::Permutations::new(charset, min, max)?;
            Ok(Box::new(it))
        }
        Expression::Glob { pattern } => {
            let it = glob::Glob::new(pattern)?;
            Ok(Box::new(it))
        }
        Expression::Range { min, max, set } => {
            let it = range::Range::new(min, max, set)?;
            Ok(Box::new(it))
        }
        Expression::Multiple { expressions } => {
            let mut iters = vec![];
            for expr in expressions.iter() {
                iters.push(new(expr.clone())?)
            }

            let it = multi::Multi::new(iters)?;

            Ok(Box::new(it))
        }
    }
}
