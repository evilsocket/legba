use std::collections::HashMap;

use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;

use crate::session::Error;

const CONTEXT_EXPRESSION_ERROR: &str =
    "context expression must be in the form of KEY1=VALUE1&KEY2=VALUE2&...";

static USER_CONTEXT_PARSER: Lazy<Regex> = lazy_regex!(r"(?m)&?([^&]+)=([^&]+)");

#[derive(Default)]
pub(crate) struct Context {
    data: HashMap<String, String>,
}

impl Context {
    pub fn parse(expr: &str) -> Result<Self, Error> {
        let mut context = Self::default();

        for cap in USER_CONTEXT_PARSER.captures_iter(expr) {
            let key = cap.get(1).ok_or(CONTEXT_EXPRESSION_ERROR)?.as_str();
            let value = cap.get(2).ok_or(CONTEXT_EXPRESSION_ERROR)?.as_str();

            context.add(key, value);
        }

        if !expr.is_empty() && context.data.is_empty() {
            Err(CONTEXT_EXPRESSION_ERROR.to_owned())
        } else {
            Ok(context)
        }
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a str> {
        if let Some(val) = self.data.get(key) {
            Some(val)
        } else {
            None
        }
    }

    pub fn add(&mut self, key: &str, val: &str) {
        self.data.insert(key.to_owned(), val.to_owned());
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, String> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::CONTEXT_EXPRESSION_ERROR;
    use super::Context;

    #[test]
    fn wont_parse_without_value() {
        let ctx = Context::parse("foo=");
        assert_eq!(ctx.err(), Some(CONTEXT_EXPRESSION_ERROR.to_owned()));
    }

    #[test]
    fn wont_parse_without_key() {
        let ctx = Context::parse("=bar");
        assert_eq!(ctx.err(), Some(CONTEXT_EXPRESSION_ERROR.to_owned()));
    }

    #[test]
    fn can_parse_nothing() {
        let ctx = Context::parse("").unwrap();
        assert_eq!(ctx.get("foo"), None);
    }

    #[test]
    fn can_parse_single_pair() {
        let ctx = Context::parse("foo=bar").unwrap();
        assert_eq!(ctx.get("foo"), Some("bar"));
    }

    #[test]
    fn can_parse_single_pair_with_ampersand_prefix() {
        let ctx = Context::parse("&foo=bar").unwrap();
        assert_eq!(ctx.get("foo"), Some("bar"));
    }

    #[test]
    fn can_parse_single_pair_with_ampersand_suffix() {
        let ctx = Context::parse("foo=bar&").unwrap();
        assert_eq!(ctx.get("foo"), Some("bar"));
    }

    #[test]
    fn can_parse_single_pair_with_ampersands() {
        let ctx = Context::parse("&foo=bar&").unwrap();
        assert_eq!(ctx.get("foo"), Some("bar"));
    }

    #[test]
    fn can_parse_multiple_pairs() {
        let ctx = Context::parse("foo=bar&moo=tar").unwrap();
        assert_eq!(ctx.get("foo"), Some("bar"));
        assert_eq!(ctx.get("moo"), Some("tar"));
    }

    #[test]
    fn can_parse_multiple_pairs_with_spaces_and_stuff() {
        let ctx = Context::parse("foo=bar bor&moo=tar||tor/ ur").unwrap();
        assert_eq!(ctx.get("foo"), Some("bar bor"));
        assert_eq!(ctx.get("moo"), Some("tar||tor/ ur"));
    }
}
