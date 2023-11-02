use std::time;

use crate::{
    creds::{self, expression, iterator, Credentials},
    options::Options,
    session::Error,
};

use super::Expression;

pub(crate) struct Combinator {
    options: Options,
    user_expr: creds::Expression,
    user_it: Box<dyn creds::Iterator>,
    current_user: Option<String>,
    pass_it: Box<dyn creds::Iterator>,
    pass_expr: creds::Expression,
    dispatched: usize,
    search_space_size: usize,
    single: bool,
}

impl Combinator {
    pub fn from_options(options: Options, from: usize, single: bool) -> Result<Self, Error> {
        let (user_expr, user_it, pass_expr, pass_it) = if single {
            // get either username or password
            let user_expr = if options.username.is_some() {
                expression::parse_expression(options.username.as_ref())
            } else {
                expression::parse_expression(options.password.as_ref())
            };
            let user_it = iterator::new(user_expr.clone())?;

            (
                user_expr,
                user_it,
                creds::Expression::default(),
                iterator::empty()?,
            )
        } else {
            // get both
            let user_expr = expression::parse_expression(options.username.as_ref());
            let user_it = iterator::new(user_expr.clone())?;

            let expr = expression::parse_expression(options.password.as_ref());
            (
                user_expr,
                user_it,
                expr.clone(),
                iterator::new(expr.clone())?,
            )
        };

        let search_space_size =
            user_it.search_space_size() * std::cmp::max(pass_it.search_space_size(), 1);
        let dispatched = 0;
        let current_user = None;
        let mut combinator = Self {
            user_expr,
            user_it,
            pass_it,
            pass_expr,
            options,
            search_space_size,
            single,
            dispatched,
            current_user,
        };

        // restore from last state if needed
        if from > 0 {
            let start = time::Instant::now();
            while combinator.dispatched < from {
                let _ = combinator.next();
            }
            log::info!("restored from credential {} in {:?}", from, start.elapsed());
        }

        Ok(combinator)
    }

    pub fn from_plugin_override(
        expression: Expression,
        from: usize,
        options: Options,
    ) -> Result<Self, Error> {
        let pass_expr = creds::Expression::default();
        let pass_it = iterator::empty()?;
        let payload_it = iterator::new(expression.clone())?;
        let search_space_size = payload_it.search_space_size();
        let dispatched = 0;
        let current_user = None;
        let mut combinator = Self {
            user_expr: expression,
            user_it: payload_it,
            pass_it,
            pass_expr,
            options,
            search_space_size,
            single: true,
            dispatched,
            current_user,
        };

        // restore from last state if needed
        if from > 0 {
            let start = time::Instant::now();
            while combinator.dispatched < from {
                let _ = combinator.next();
            }
            log::info!("restored from credential {} in {:?}", from, start.elapsed());
        }

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

    fn get_next_pass(&mut self) -> (bool, String) {
        if self.single {
            // single credentials mode
            (false, "".to_owned())
        } else if let Some(next_pass) = self.pass_it.next() {
            // return next password
            (false, next_pass)
        } else {
            // reset internal iterator
            self.pass_it = iterator::new(self.pass_expr.clone()).unwrap();
            (true, self.pass_it.next().unwrap())
        }
    }
}

impl Iterator for Combinator {
    type Item = Credentials;

    fn next(&mut self) -> Option<Self::Item> {
        // we're done
        if self.dispatched == self.search_space_size {
            return None;
        }

        let (is_reset, next_pass) = self.get_next_pass();
        // if we're in the initial state, or single mode, or the password iterator just completed and resetted,
        // get the next user. this simulates a nested iteration such as:
        //
        //  for user in users {
        //      for pass in passwords {
        //          ...
        //      }
        //  }
        if self.single || self.dispatched == 0 || is_reset {
            self.current_user = self.user_it.next();
        }

        // check if we have to rate limit
        if self.options.rate_limit > 0 && self.dispatched % self.options.rate_limit == 0 {
            std::thread::sleep(time::Duration::from_secs(1));
        }

        self.dispatched += 1;

        Some(Credentials {
            username: self.current_user.as_ref().unwrap().to_owned(),
            password: next_pass,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::creds::{Credentials, Expression};

    use super::Combinator;

    #[test]
    fn returns_plugin_overrides_min_max() {
        let expr = Expression::Range {
            min: 1,
            max: 10,
            set: vec![],
        };
        let opts = crate::Options::default();
        let comb = Combinator::from_plugin_override(expr, 0, opts).unwrap();
        let mut expected = vec![];
        let mut got = vec![];

        for i in 1..=10 {
            expected.push(Credentials {
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
        let comb = Combinator::from_plugin_override(expr, 0, opts).unwrap();
        let mut expected = vec![];
        let mut got = vec![];

        for i in set {
            expected.push(Credentials {
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
                    username: format!("user{}", i),
                    password: format!("pass{}", j),
                })
            }
        }
        let mut opts = crate::Options::default();
        opts.username = Some(tmpuserspath.to_str().unwrap().to_owned());
        opts.password = Some(tmppasspath.to_str().unwrap().to_owned());

        let comb = Combinator::from_options(opts, 0, false).unwrap();
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
                username: format!("test{}", i),
                password: "".to_owned(),
            })
        }

        tmpdata.flush().unwrap();
        drop(tmpdata);

        let mut opts = crate::Options::default();
        opts.username = Some(tmppath.to_str().unwrap().to_owned());

        let comb = Combinator::from_options(opts, 0, true).unwrap();
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
}
