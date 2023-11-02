use crate::creds::expression::Expression;
use crate::session::Error;

mod constant;
mod empty;
mod glob;
mod permutations;
mod permutator;
mod range;
mod wordlist;

pub(crate) trait Iterator: std::iter::Iterator<Item = String> {
    fn search_space_size(&self) -> usize;
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
    }
}

pub(crate) fn empty() -> Result<Box<dyn Iterator>, Error> {
    Ok(Box::new(empty::Empty::new()))
}
