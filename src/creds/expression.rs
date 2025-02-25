use std::fmt;
use std::path::Path;

use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;
use serde::Serialize;

const DEFAULT_PERMUTATIONS_MIN_LEN: usize = 4;
const DEFAULT_PERMUTATIONS_MAX_LEN: usize = 8;
const DEFAULT_PERMUTATIONS_CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_ !\"#$%&\'()*+,-./:;<=>?@[\\]^`{|}~";

static PERMUTATIONS_PARSER: Lazy<Regex> = lazy_regex!(r"^#(\d+)-(\d+)(:.+)?$");
static RANGE_MIN_MAX_PARSER: Lazy<Regex> = lazy_regex!(r"^\[(\d+)-(\d+)\]$");
static RANGE_SET_PARSER: Lazy<Regex> = lazy_regex!(r"^\[(\d+(,\s*\d+)*)?\]$");

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) enum Expression {
    Constant {
        value: String,
    },
    Wordlist {
        filename: String,
    },
    Permutations {
        min: usize,
        max: usize,
        charset: String,
    },
    Range {
        min: usize,
        max: usize,
        set: Vec<usize>,
    },
    Glob {
        pattern: String,
    },
    Multiple {
        expressions: Vec<Expression>,
    },
}

impl Expression {
    pub fn as_string(&self) -> String {
        match self {
            Expression::Constant { value } => value.to_owned(),
            Expression::Wordlist { filename } => filename.to_owned(),
            Expression::Permutations { min, max, charset } => {
                format!("#{min}-{max}:{charset}")
            }
            Expression::Range { min, max, set } => {
                if set.is_empty() {
                    format!("[{min}-{max}]")
                } else {
                    format!(
                        "[{}]",
                        set.iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                    )
                }
            }
            Expression::Glob { pattern } => format!("@{pattern}"),
            Expression::Multiple { expressions } => expressions
                .iter()
                .map(|e| e.as_string())
                .collect::<Vec<String>>()
                .join(", "),
        }
    }

    pub fn is_default(&self) -> bool {
        self == &Expression::default()
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Permutations {
            min: DEFAULT_PERMUTATIONS_MIN_LEN,
            max: DEFAULT_PERMUTATIONS_MAX_LEN,
            charset: DEFAULT_PERMUTATIONS_CHARSET.to_owned(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Constant { value } => write!(f, "string '{}'", value),
            Expression::Wordlist { filename } => write!(f, "wordlist {}", filename),
            Expression::Permutations { min, max, charset } => {
                write!(
                    f,
                    "permutations (min:{} max:{} charset:{})",
                    min, max, charset
                )
            }
            Expression::Glob { pattern } => write!(f, "glob {}", pattern),
            Expression::Range { min, max, set } => {
                if set.is_empty() {
                    write!(f, "range {} -> {}", min, max)
                } else {
                    write!(f, "range {:?}", set)
                }
            }
            Expression::Multiple { expressions: _ } => write!(f, "multi {{ ... }}"),
        }
    }
}

pub(crate) fn parse_expression(expr: Option<&String>) -> Expression {
    if let Some(expr) = expr {
        match expr.chars().next().unwrap_or(' ') {
            // permutations or constant
            '#' => {
                // permutations expression
                if let Some(captures) = PERMUTATIONS_PARSER.captures(expr) {
                    if captures.get(3).is_some() {
                        // with custom charset
                        return Expression::Permutations {
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
                        return Expression::Permutations {
                            min: captures.get(1).unwrap().as_str().parse().unwrap(),
                            max: captures.get(2).unwrap().as_str().parse().unwrap(),
                            charset: DEFAULT_PERMUTATIONS_CHARSET.to_owned(),
                        };
                    }
                }

                // constant value casually starting with #
                return Expression::Constant {
                    value: expr.to_owned(),
                };
            }
            // glob expression or constant
            '@' => {
                return if expr.contains('*') {
                    // in order to be considered a glob expression at least one * must be used
                    // constant value casually starting with @
                    Expression::Glob {
                        pattern: expr[1..].to_owned(),
                    }
                } else {
                    // constant value casually starting with @
                    Expression::Constant {
                        value: expr.to_owned(),
                    }
                };
            }
            // range expression or constant
            '[' => {
                return if let Some(captures) = RANGE_MIN_MAX_PARSER.captures(expr) {
                    // [min-max]
                    Expression::Range {
                        min: captures.get(1).unwrap().as_str().parse().unwrap(),
                        max: captures.get(2).unwrap().as_str().parse().unwrap(),
                        set: vec![],
                    }
                } else if let Some(captures) = RANGE_SET_PARSER.captures(expr) {
                    // [n, n, n, ...]
                    Expression::Range {
                        min: 0,
                        max: 0,
                        set: captures
                            .get(1)
                            .unwrap()
                            .as_str()
                            .split(',')
                            .map(|s| s.trim().parse().unwrap())
                            .collect(),
                    }
                } else {
                    // constant value casually starting with [
                    Expression::Constant {
                        value: expr.to_owned(),
                    }
                };
            }
            // file name, constant or multiple
            _ => {
                let filepath = Path::new(&expr);
                if filepath.exists() && filepath.is_file() {
                    // this is a file name
                    return Expression::Wordlist {
                        filename: expr.to_owned(),
                    };
                } else if expr.contains(',') {
                    // parse as multiple expressions
                    let multi = expr
                        .split(',')
                        .map(|s| s.trim().to_owned())
                        .collect::<Vec<String>>();
                    let mut expressions = vec![];
                    for exp in multi {
                        expressions.push(parse_expression(Some(exp).as_ref()));
                    }

                    return Expression::Multiple { expressions };
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
    use super::DEFAULT_PERMUTATIONS_CHARSET;
    use super::DEFAULT_PERMUTATIONS_MAX_LEN;
    use super::DEFAULT_PERMUTATIONS_MIN_LEN;
    use super::Expression;
    use super::parse_expression;

    #[test]
    fn can_parse_none() {
        let res = parse_expression(None);
        assert_eq!(
            res,
            Expression::Permutations {
                min: DEFAULT_PERMUTATIONS_MIN_LEN,
                max: DEFAULT_PERMUTATIONS_MAX_LEN,
                charset: DEFAULT_PERMUTATIONS_CHARSET.to_owned(),
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
    fn can_parse_constant_with_bracket() {
        let res = parse_expression(Some("[m_n0t_@_range]".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Constant {
                value: "[m_n0t_@_range]".to_owned()
            }
        )
    }

    #[test]
    fn can_parse_permutations_with_default_charset() {
        let res = parse_expression(Some("#1-3".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Permutations {
                min: 1,
                max: 3,
                charset: DEFAULT_PERMUTATIONS_CHARSET.to_owned(),
            }
        )
    }

    #[test]
    fn can_parse_permutations_with_custom_charset() {
        let res = parse_expression(Some("#1-10:abcdef".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Permutations {
                min: 1,
                max: 10,
                charset: "abcdef".to_owned(),
            }
        )
    }

    #[test]
    fn can_parse_range_with_min_max() {
        let res = parse_expression(Some("[1-3]".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Range {
                min: 1,
                max: 3,
                set: vec![],
            }
        )
    }

    #[test]
    fn can_parse_range_with_set() {
        let res = parse_expression(Some("[1,3,4, 5, 6, 7, 8, 12,666]".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Range {
                min: 0,
                max: 0,
                set: vec![1, 3, 4, 5, 6, 7, 8, 12, 666],
            }
        )
    }

    #[test]
    fn can_parse_glob() {
        let res = parse_expression(Some("@/etc/*".to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Glob {
                pattern: "/etc/*".to_owned()
            }
        )
    }

    #[test]
    fn can_parse_multiple() {
        let expr = "1,[3-5],[6-8],9,[10-13]";
        let res = parse_expression(Some(expr.to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Multiple {
                expressions: vec![
                    Expression::Constant {
                        value: "1".to_string()
                    },
                    Expression::Range {
                        min: 3,
                        max: 5,
                        set: vec![],
                    },
                    Expression::Range {
                        min: 6,
                        max: 8,
                        set: vec![],
                    },
                    Expression::Constant {
                        value: "9".to_string()
                    },
                    Expression::Range {
                        min: 10,
                        max: 13,
                        set: vec![],
                    },
                ]
            }
        )
    }

    #[test]
    fn can_parse_multiple_with_spaces() {
        let expr = "1, [3-4], 9 ";
        let res = parse_expression(Some(expr.to_owned()).as_ref());
        assert_eq!(
            res,
            Expression::Multiple {
                expressions: vec![
                    Expression::Constant {
                        value: "1".to_string()
                    },
                    Expression::Range {
                        min: 3,
                        max: 4,
                        set: vec![],
                    },
                    Expression::Constant {
                        value: "9".to_string()
                    },
                ]
            }
        )
    }
}
