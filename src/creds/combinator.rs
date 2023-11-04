use std::time;

use clap::ValueEnum;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    creds::{self, expression, iterator, Credentials},
    options::Options,
    session::Error,
};

use super::Expression;

#[derive(ValueEnum, Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) enum IterationStrategy {
    #[default]
    User,
    Password,
}

pub(crate) struct Combinator {
    options: Options,

    user_expr: creds::Expression,
    pass_expr: creds::Expression,
    product: Box<dyn Iterator<Item = (String, String, String)>>,

    dispatched: usize,
    search_space_size: usize,
}

impl Combinator {
    fn reset_from(&mut self, from: usize) {
        if from > 0 {
            let start = time::Instant::now();
            while self.dispatched < from {
                let _ = self.next();
            }
            log::info!("restored from credential {} in {:?}", from, start.elapsed());
        }
    }

    fn combine_iterators(
        options: &Options,
        targets: Vec<String>,
        user_it: Box<dyn creds::Iterator>,
        pass_it: Option<Box<dyn creds::Iterator>>,
    ) -> Box<dyn Iterator<Item = (String, String, String)>> {
        if let Some(pass_it) = pass_it {
            let (outer, inner) = match options.iterate_by {
                IterationStrategy::User => (user_it, pass_it),
                IterationStrategy::Password => (pass_it, user_it),
            };

            Box::new(
                targets
                    .into_iter()
                    .cartesian_product(outer)
                    .cartesian_product(inner)
                    .map(|((t, out), inn)| (t.to_owned(), out, inn)),
            )
        } else {
            Box::new(
                targets
                    .into_iter()
                    .cartesian_product(user_it)
                    .map(|(t, payload)| (t.to_owned(), payload, "".to_owned())),
            )
        }
    }

    fn for_single_payload(
        targets: &Vec<String>,
        options: Options,
        override_expr: Option<Expression>,
    ) -> Result<Self, Error> {
        let dispatched = 0;
        // get either override, username or password
        let payload_expr = if let Some(override_expr) = override_expr {
            override_expr
        } else if options.username.is_some() {
            expression::parse_expression(options.username.as_ref())
        } else {
            expression::parse_expression(options.password.as_ref())
        };
        let payload_it = iterator::new(payload_expr.clone())?;
        let search_space_size: usize = targets.len() * payload_it.search_space_size();
        let product = Self::combine_iterators(&options, targets.to_owned(), payload_it, None);

        Ok(Self {
            options,
            user_expr: payload_expr,
            pass_expr: creds::Expression::default(),
            product,
            search_space_size,
            dispatched,
        })
    }

    fn for_double_payload(targets: &Vec<String>, options: Options) -> Result<Self, Error> {
        let dispatched = 0;
        // get both
        let user_expr = expression::parse_expression(options.username.as_ref());
        let user_it = iterator::new(user_expr.clone())?;
        let pass_expr = expression::parse_expression(options.password.as_ref());
        let pass_it = iterator::new(pass_expr.clone())?;
        let search_space_size =
            targets.len() * user_it.search_space_size() * pass_it.search_space_size();
        let product = Self::combine_iterators(&options, targets.to_owned(), user_it, Some(pass_it));

        Ok(Self {
            options,
            user_expr,
            pass_expr,
            product,
            search_space_size,
            dispatched,
        })
    }

    pub fn create(
        targets: &Vec<String>,
        options: Options,
        from: usize,
        single: bool,
        override_expression: Option<Expression>,
    ) -> Result<Self, Error> {
        let mut combinator = if single {
            Self::for_single_payload(targets, options, override_expression)?
        } else {
            Self::for_double_payload(targets, options)?
        };

        // restore from last state if needed
        combinator.reset_from(from);

        Ok(combinator)
    }

    pub fn search_space_size(&self) -> usize {
        self.search_space_size
    }

    pub fn username_expression(&self) -> &creds::Expression {
        &self.user_expr
    }

    pub fn password_expression(&self) -> &creds::Expression {
        &self.pass_expr
    }
}

impl Iterator for Combinator {
    // (target, credentials)
    type Item = Credentials;

    fn next(&mut self) -> Option<Self::Item> {
        // we're done
        if let Some((target, outer, inner)) = self.product.next() {
            // check if we have to rate limit
            if self.options.rate_limit > 0 && self.dispatched % self.options.rate_limit == 0 {
                std::thread::sleep(time::Duration::from_secs(1));
            }

            let (username, password) = match self.options.iterate_by {
                IterationStrategy::User => (outer, inner),
                IterationStrategy::Password => (inner, outer),
            };

            Some(Credentials {
                target,
                username,
                password,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::creds::{Credentials, Expression, IterationStrategy};

    use super::Combinator;

    #[test]
    fn can_handle_user_iteration_strategy() {
        let targets = vec!["foo".to_owned()];
        let mut opts = crate::Options::default();

        opts.iterate_by = IterationStrategy::User; // default
        opts.username = Some("#1-2:u".to_owned());
        opts.password = Some("#1-2:p".to_owned());

        let comb = Combinator::create(&targets, opts, 0, false, None).unwrap();
        let expected = vec![
            Credentials {
                target: "foo".to_owned(),
                username: "u".to_owned(),
                password: "p".to_owned(),
            },
            Credentials {
                target: "foo".to_owned(),
                username: "u".to_owned(),
                password: "pp".to_owned(),
            },
            Credentials {
                target: "foo".to_owned(),
                username: "uu".to_owned(),
                password: "p".to_owned(),
            },
            Credentials {
                target: "foo".to_owned(),
                username: "uu".to_owned(),
                password: "pp".to_owned(),
            },
        ];
        let mut got = vec![];
        for cred in comb {
            got.push(cred);
        }

        assert_eq!(expected, got);
    }

    #[test]
    fn can_handle_password_iteration_strategy() {
        let targets = vec!["foo".to_owned()];
        let mut opts = crate::Options::default();

        opts.iterate_by = IterationStrategy::Password;
        opts.username = Some("#1-2:u".to_owned());
        opts.password = Some("#1-2:p".to_owned());

        let comb = Combinator::create(&targets, opts, 0, false, None).unwrap();
        let expected = vec![
            Credentials {
                target: "foo".to_owned(),
                username: "u".to_owned(),
                password: "p".to_owned(),
            },
            Credentials {
                target: "foo".to_owned(),
                username: "uu".to_owned(),
                password: "p".to_owned(),
            },
            Credentials {
                target: "foo".to_owned(),
                username: "u".to_owned(),
                password: "pp".to_owned(),
            },
            Credentials {
                target: "foo".to_owned(),
                username: "uu".to_owned(),
                password: "pp".to_owned(),
            },
        ];
        let mut got = vec![];
        for cred in comb {
            got.push(cred);
        }

        assert_eq!(expected, got);
    }

    #[test]
    fn iteration_strategies_return_same_results() {
        let targets = vec!["foo".to_owned()];

        let mut by_user_opts = crate::Options::default();
        by_user_opts.iterate_by = IterationStrategy::User;
        by_user_opts.username = Some("#1-2:u".to_owned());
        by_user_opts.password = Some("#1-5:p".to_owned());

        let mut by_pass_opts = crate::Options::default();
        by_pass_opts.iterate_by = IterationStrategy::Password;
        by_pass_opts.username = Some("#1-2:u".to_owned());
        by_pass_opts.password = Some("#1-5:p".to_owned());

        let by_user_comb = Combinator::create(&targets, by_user_opts, 0, false, None).unwrap();
        let by_pass_comb = Combinator::create(&targets, by_pass_opts, 0, false, None).unwrap();

        assert_eq!(
            by_user_comb.search_space_size(),
            by_pass_comb.search_space_size()
        );

        let mut by_user: Vec<Credentials> = by_user_comb.collect();
        let mut by_pass: Vec<Credentials> = by_pass_comb.collect();

        by_user.sort_by(|a, b| a.partial_cmp(b).unwrap());
        by_pass.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(by_user, by_pass);
    }

    #[test]
    fn can_handle_multiple_targets_and_double_credentials() {
        let targets = vec!["foo".to_owned(), "bar".to_owned()];
        let mut opts = crate::Options::default();

        opts.username = Some("[1, 2, 3]".to_owned());
        opts.password = Some("[1, 2, 3]".to_owned());

        let comb = Combinator::create(&targets, opts, 0, false, None).unwrap();
        let mut expected = vec![];
        let mut got = vec![];

        for t in targets {
            for u in 1..=3 {
                for p in 1..=3 {
                    expected.push(Credentials {
                        target: t.to_owned(),
                        username: u.to_string(),
                        password: p.to_string(),
                    });
                }
            }
        }

        for cred in comb {
            got.push(cred);
        }

        assert_eq!(expected, got);
    }

    #[test]
    fn can_handle_multiple_targets_and_single_credentials() {
        let targets = vec!["foo".to_owned(), "bar".to_owned()];
        let mut opts = crate::Options::default();

        opts.username = Some("[1, 2, 3]".to_owned());

        let comb = Combinator::create(&targets, opts, 0, true, None).unwrap();
        let mut expected = vec![];
        let mut got = vec![];

        for t in targets {
            for u in 1..=3 {
                expected.push(Credentials {
                    target: t.to_owned(),
                    username: u.to_string(),
                    password: "".to_string(),
                });
            }
        }

        for cred in comb {
            got.push(cred);
        }

        assert_eq!(expected, got);
    }

    #[test]
    fn returns_plugin_overrides_min_max() {
        let expr = Expression::Range {
            min: 1,
            max: 10,
            set: vec![],
        };
        let opts = crate::Options::default();
        let comb = Combinator::create(&vec!["foo".to_owned()], opts, 0, true, Some(expr)).unwrap();
        let mut expected = vec![];
        let mut got = vec![];

        for i in 1..=10 {
            expected.push(Credentials {
                target: "foo".to_owned(),
                username: i.to_string(),
                password: "".to_owned(),
            });
        }

        for cred in comb {
            got.push(cred);
        }

        assert_eq!(expected, got);
    }

    #[test]
    fn returns_plugin_overrides_set() {
        let set = vec![5, 12, 777, 666];
        let expr = Expression::Range {
            min: 0,
            max: 0,
            set: set.clone(),
        };
        let opts = crate::Options::default();
        let comb = Combinator::create(&vec!["foo".to_owned()], opts, 0, true, Some(expr)).unwrap();
        let mut expected = vec![];
        let mut got = vec![];

        for i in set {
            expected.push(Credentials {
                target: "foo".to_owned(),
                username: i.to_string(),
                password: "".to_owned(),
            });
        }

        for cred in comb {
            got.push(cred);
        }

        assert_eq!(expected, got);
    }

    #[test]
    fn returns_all_combinations_of_two_wordlists() {
        let num_items = 123;
        let mut expected = vec![];
        let tmpdir = tempfile::tempdir().unwrap();
        let tmpuserspath = tmpdir.path().join("users.txt");
        let tmppasspath = tmpdir.path().join("passwords.txt");
        let mut tmpusers = File::create(&tmpuserspath).unwrap();
        let mut tmppasswords = File::create(&tmppasspath).unwrap();

        for i in 0..num_items {
            write!(tmpusers, "user{}\n", i).unwrap();
            write!(tmppasswords, "pass{}\n", i).unwrap();
        }

        tmpusers.flush().unwrap();
        drop(tmpusers);
        tmppasswords.flush().unwrap();
        drop(tmppasswords);

        for i in 0..num_items {
            for j in 0..num_items {
                expected.push(Credentials {
                    target: "foo".to_owned(),
                    username: format!("user{}", i),
                    password: format!("pass{}", j),
                })
            }
        }
        let mut opts = crate::Options::default();
        opts.username = Some(tmpuserspath.to_str().unwrap().to_owned());
        opts.password = Some(tmppasspath.to_str().unwrap().to_owned());

        let comb = Combinator::create(&vec!["foo".to_owned()], opts, 0, false, None).unwrap();
        let tot = comb.search_space_size();
        let mut got = vec![];

        for cred in comb {
            got.push(cred);
        }

        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(expected.len(), tot);
        assert_eq!(got.len(), tot);

        assert_eq!(expected, got);
    }

    #[test]
    fn returns_all_elements_of_one_wordlist() {
        let num_items = 123;
        let mut expected = vec![];
        let tmpdir = tempfile::tempdir().unwrap();
        let tmppath = tmpdir.path().join("list.txt");
        let mut tmpdata = File::create(&tmppath).unwrap();

        for i in 0..num_items {
            write!(tmpdata, "test{}\n", i).unwrap();
            expected.push(Credentials {
                target: "foo".to_owned(),
                username: format!("test{}", i),
                password: "".to_owned(),
            });
        }

        tmpdata.flush().unwrap();
        drop(tmpdata);

        let mut opts = crate::Options::default();
        opts.username = Some(tmppath.to_str().unwrap().to_owned());

        let comb = Combinator::create(&vec!["foo".to_owned()], opts, 0, true, None).unwrap();
        let tot = comb.search_space_size();
        assert_eq!(expected.len(), tot);

        let mut got = vec![];
        for cred in comb {
            got.push(cred);
        }

        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(got.len(), tot);
        assert_eq!(expected, got);
    }
}
