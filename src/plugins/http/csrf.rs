use std::time::Duration;

use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use url::Url;

use crate::session::Error;

#[derive(Clone)]
pub(crate) struct Config {
    page: String,
    re: Regex,
}

impl Config {
    pub fn new(page: &str, re: &str) -> Result<Self, Error> {
        let re = Regex::new(re).map_err(|e| e.to_string())?;
        // first group = whole string
        // second group = token name
        // third group = token value
        if re.captures_len() != 3 {
            return Err(format!(
                "csrf expression expected to have 2 capture groups, got {}",
                re.captures_len() - 1
            ));
        }

        let page = Url::parse(page).map_err(|e| e.to_string())?.to_string();
        let re = re;

        Ok(Self { page, re })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Token {
    pub name: String,
    pub value: String,
    pub cookie: String,
}

pub(crate) async fn handle(
    config: &Config,
    client: Client,
    headers: HeaderMap<HeaderValue>,
    timeout: Duration,
) -> Result<Option<Token>, Error> {
    let mut token = Token::default();
    match client
        .get(&config.page)
        .headers(headers)
        .timeout(timeout)
        .send()
        .await
    {
        Err(e) => {
            log::debug!("error requesting csrf token from {}: {:?}", config.page, e);
            Err(e.to_string())
        }
        Ok(res) => {
            if res.status().is_success() {
                // get cookie from header
                if let Some(cookie) = res.headers().get("set-cookie") {
                    token.cookie = cookie.to_str().unwrap().to_owned();
                } else {
                    log::warn!("csrf page unexpectetly did not return any cookie");
                }

                let body = res.text().await;
                if let Ok(body) = body {
                    if let Some(captures) = config.re.captures(&body) {
                        if captures.len() == 3 {
                            token.name = captures.get(1).unwrap().as_str().to_owned();
                            token.value = captures.get(2).unwrap().as_str().to_owned();
                            log::debug!("{:?}", &token);
                            Ok(Some(token))
                        } else {
                            log::error!(
                                "csrf expression expected to have 2 capture groups, got {}",
                                captures.len() - 1
                            );
                            Ok(None)
                        }
                    } else {
                        log::error!("csrf expression could not capture any token");
                        Ok(None)
                    }
                } else {
                    let err = body.err().unwrap().to_string();
                    log::error!("error fetching csrf page body: {}", &err);
                    Err(err)
                }
            } else {
                log::error!("csrf token page returned status: {:?}", res.status());
                Ok(None)
            }
        }
    }
}
