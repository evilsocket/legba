use crate::creds::expression::Expression;
use crate::session::Error;

mod constant;
mod empty;
mod glob;
mod permutator;
mod range;
mod wordlist;

pub(crate) trait Iterator: std::iter::Iterator<Item = String> {
    fn search_space_size(&self) -> usize;
}

pub(crate) fn new(expr: Expression) -> Result<Box<dyn Iterator>, Error> {
    match expr {
        Expression::Constant { value } => {
            let gen = constant::Constant::new(value)?;
            Ok(Box::new(gen))
        }
        Expression::Wordlist { filename } => {
            let gen = wordlist::Wordlist::new(filename)?;
            Ok(Box::new(gen))
        }
        Expression::Range { min, max, charset } => {
            let gen = range::Range::new(charset, min, max)?;
            Ok(Box::new(gen))
        }
        Expression::Glob { pattern } => {
            let gen = glob::Glob::new(pattern)?;
            Ok(Box::new(gen))
        }
    }
}

pub(crate) fn empty() -> Result<Box<dyn Iterator>, Error> {
    Ok(Box::new(empty::Empty::new()))
}
