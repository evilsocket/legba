use std::fmt;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

const DEFAULT_MIN_LEN: usize = 4;
const DEFAULT_MAX_LEN: usize = 8;
const DEFAULT_CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_ !\"#$%&\'()*+,-./:;<=>?@[\\]^`{|}~";

lazy_static! {
    static ref RANGE_PARSER: Regex = Regex::new(r"^#(\d+)-(\d+)(:.+)?$").unwrap();
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expression {
    Constant {
        value: String,
    },
    Wordlist {
        filename: String,
    },
    Range {
        min: usize,
        max: usize,
        charset: String,
    },
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Range {
            min: DEFAULT_MIN_LEN,
            max: DEFAULT_MAX_LEN,
            charset: DEFAULT_CHARSET.to_owned(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Constant { value } => write!(f, "string '{}'", value),
            Expression::Wordlist { filename } => write!(f, "wordlist {}", filename),
            Expression::Range { min, max, charset } => {
                write!(f, "range (min:{} max:{} charset:{})", min, max, charset)
            }
        }
    }
}

pub(crate) fn parse_expression(expr: Option<&String>) -> Expression {
    if let Some(expr) = expr {
        match expr.chars().next().unwrap_or(' ') {
            // range or constant
            '#' => {
                // range expression
                if let Some(captures) = RANGE_PARSER.captures(expr) {
                    if captures.get(3).is_some() {
                        // with custom charset
                        return Expression::Range {
                            min: captures.get(1).unwrap().as_str().parse().unwrap(),
                            max: captures.get(2).unwrap().as_str().parse().unwrap(),
                            charset: captures
                                .get(3)
                                .unwrap()
                                .as_str()
                                .strip_prefix(':')
                                .unwrap()
                                .to_owned(),
                        };
                    } else {
                        // with default charset
                        return Expression::Range {
                            min: captures.get(1).unwrap().as_str().parse().unwrap(),
                            max: captures.get(2).unwrap().as_str().parse().unwrap(),
                            charset: DEFAULT_CHARSET.to_owned(),
                        };
                    }
                }

                // constant value casually starting with #
                return Expression::Constant {
                    value: expr.to_owned(),
                };
            }
            // file name or constant
            _ => {
                let filepath = Path::new(&expr);
                if filepath.exists() {
                    // this is a file name
                    return Expression::Wordlist {
                        filename: expr.to_owned(),
                    };
                } else {
                    // constant value casually starting with @
                    return Expression::Constant {
                        value: expr.to_owned(),
                    };
                }
            }
        };
    }

    Expression::default()
}

#[cfg(test)]
mod tests {
    use super::parse_expression;
    use super::Expression;
    use super::DEFAULT_CHARSET;
    use super::DEFAULT_MAX_LEN;
    use super::DEFAULT_MIN_LEN;

    #[test]
    fn can_parse_none() {
        let res = parse_expression(None);
        assert_eq!(
            res,
            Expression::Range {
                min: DEFAULT_MIN_LEN,
                max: DEFAULT_MAX_LEN,
                charset: DEFAULT_CHARSET.to_owned(),
            }
        )
    }

    #[test]
    fn can_parse_constant() {
        let res = parse_expression(Some("admin".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Constant {
                value: "admin".to_owned()
            }
        )
    }

    #[test]
    fn can_parse_filename() {
        let res = parse_expression(Some("/etc/hosts".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Wordlist {
                filename: "/etc/hosts".to_owned()
            }
        )
    }

    #[test]
    fn can_parse_constant_with_at() {
        let res = parse_expression(Some("@m_n0t_@_f1l3".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Constant {
                value: "@m_n0t_@_f1l3".to_owned()
            }
        )
    }

    #[test]
    fn can_parse_range_with_default_charset() {
        let res = parse_expression(Some("#1-3".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Range {
                min: 1,
                max: 3,
                charset: DEFAULT_CHARSET.to_owned(),
            }
        )
    }

    #[test]
    fn can_parse_range_with_custom_charset() {
        let res = parse_expression(Some("#1-10:abcdef".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Range {
                min: 1,
                max: 10,
                charset: "abcdef".to_owned(),
            }
        )
    }
}
