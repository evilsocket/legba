use std::time;

use crate::{
    creds::{self, expression, generator, Credentials},
    options::Options,
    session::Error,
};

pub(crate) struct Combinator {
    options: Options,
    user_expr: creds::Expression,
    user_it: Box<dyn creds::Generator>,
    current_user: Option<String>,
    pass_it: Box<dyn creds::Generator>,
    pass_expr: creds::Expression,
    dispatched: usize,
    total: usize,
    single: bool,
}

impl Combinator {
    pub fn create(options: Options, from: usize, single: bool) -> Result<Self, Error> {
        let (user_expr, user_it, pass_expr, pass_it) = if single {
            // get either username or password
            let user_expr = if options.username.is_some() {
                expression::parse_expression(options.username.as_ref())
            } else {
                expression::parse_expression(options.password.as_ref())
            };
            let user_it = generator::new(user_expr.clone())?;

            (
                user_expr,
                user_it,
                creds::Expression::default(),
                generator::empty()?,
            )
        } else {
            // get both
            let user_expr = expression::parse_expression(options.username.as_ref());
            let user_it = generator::new(user_expr.clone())?;

            let expr = expression::parse_expression(options.password.as_ref());
            (
                user_expr,
                user_it,
                expr.clone(),
                generator::new(expr.clone())?,
            )
        };

        let total = user_it.search_space_size() * std::cmp::max(pass_it.search_space_size(), 1);
        let mut combinator = Self {
            user_expr,
            user_it,
            pass_it,
            pass_expr,
            options,
            total,
            single,
            dispatched: 0,
            current_user: None,
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
        self.total
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
            self.pass_it = generator::new(self.pass_expr.clone()).unwrap();
            (true, self.pass_it.next().unwrap())
        }
    }
}

impl Iterator for Combinator {
    type Item = Credentials;

    fn next(&mut self) -> Option<Self::Item> {
        // we're done
        if self.dispatched == self.total {
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

    use crate::creds::Credentials;

    use super::Combinator;

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

        let comb = Combinator::create(opts, 0, false).unwrap();
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

        let comb = Combinator::create(opts, 0, true).unwrap();
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
