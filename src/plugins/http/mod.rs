use std::time::Duration;

use async_trait::async_trait;
use rand::seq::IndexedRandom;
use random_string;
use reqwest::{
    Client, Method, RequestBuilder, Response,
    header::{CONTENT_TYPE, COOKIE, HOST, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
    multipart, redirect,
};
use url::Url;

use crate::Options;
use crate::session::{Error, Loot};

use crate::creds::Credentials;
use crate::plugins::Plugin;

use super::plugin::PayloadStrategy;

mod csrf;
mod ntlm;
pub(crate) mod options;
mod payload;
mod placeholders;
mod ua;

// Placeholders used for interpolating --http-success-string
const HTTP_USERNAME_VAR: &str = "{$username}";
const HTTP_PASSWORD_VAR: &str = "{$password}";
const HTTP_PAYLOAD_VAR: &str = "{$payload}";

super::manager::register_plugin! {
    "http" => HTTP::new(Strategy::Request),
    "http.form" => HTTP::new(Strategy::Form),
    "http.basic" => HTTP::new(Strategy::BasicAuth),
    "http.ntlm1" => HTTP::new(Strategy::NLTMv1),
    "http.ntlm2" => HTTP::new(Strategy::NLTMv2),
    "http.enum" => HTTP::new(Strategy::Enumeration),
    "http.vhost" => HTTP::new(Strategy::VHostEnum)
}

fn method_requires_payload(method: &Method) -> bool {
    matches!(method, &Method::POST | &Method::PUT | &Method::PATCH)
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Strategy {
    Request,
    Form,
    BasicAuth,
    NLTMv1,
    NLTMv2,
    Enumeration,
    VHostEnum,
}

#[derive(Debug)]
struct Success {
    pub status: u16,
    pub content_type: String,
    pub content_length: usize,
    pub body: String,
}

#[derive(Clone)]
pub(crate) struct HTTP {
    strategy: Strategy,
    client: Client,

    csrf: Option<csrf::Config>,

    domain: String,
    workstation: String,

    real_target: Option<String>,
    user_agent: Option<String>,
    success_codes: Vec<u16>,
    success_string: Option<String>,
    failure_string: Option<String>,

    enum_ext: String,
    enum_ext_placeholder: String,

    method: Method,

    headers: HeaderMap<HeaderValue>,

    proxy: Option<String>,
    proxy_user: Option<String>,
    proxy_pass: Option<String>,

    payload: Option<String>,
}

impl HTTP {
    pub fn new(strategy: Strategy) -> Self {
        HTTP {
            strategy,
            client: Client::builder().no_proxy().build().unwrap(),
            csrf: None,
            domain: String::new(),
            workstation: String::new(),
            success_codes: vec![200],
            success_string: None,
            failure_string: None,
            enum_ext: String::new(),
            enum_ext_placeholder: String::new(),
            method: Method::GET,
            headers: HeaderMap::default(),
            user_agent: None,
            payload: None,
            proxy: None,
            proxy_user: None,
            proxy_pass: None,
            real_target: None,
        }
    }

    fn get_target_url(&self, creds: &mut Credentials) -> Result<String, Error> {
        if let Some(real_target) = self.real_target.as_ref() {
            creds.target = real_target.to_owned();
        }

        // add default schema if not present
        let target = if !creds.target.contains("://") {
            format!("https://{}", creds.target)
        } else {
            creds.target.to_owned()
        };

        // parse as url
        let target_url = Url::parse(&target).map_err(|e| e.to_string())?;
        // more logic
        let target_url = if self.strategy == Strategy::Enumeration {
            let port_part = if let Some(port) = target_url.port() {
                format!(":{}", port)
            } else {
                "".to_owned()
            };

            let path = placeholders::interpolate(target_url.path(), creds);
            let query = if let Some(query) = target_url.query() {
                format!("?{}", query)
            } else {
                "".to_owned()
            };

            format!(
                "{}://{}{}{}{}",
                target_url.scheme(),
                target_url.host().unwrap(),
                port_part,
                path,
                query
            )
        } else {
            target_url.to_string()
        };

        Ok(placeholders::interpolate(&target_url, creds))
    }

    fn setup_request_body(
        &self,
        creds: &Credentials,
        csrf: Option<csrf::Token>,
        mut builder: RequestBuilder,
    ) -> RequestBuilder {
        let mut do_body = true;
        if self.strategy == Strategy::BasicAuth {
            // set basic authentication data
            builder = builder.basic_auth(&creds.username, Some(&creds.password));
        } else if self.strategy == Strategy::Form {
            // set form data
            let fields = payload::parse_fields(self.payload.as_ref(), creds).unwrap();
            // log::info!("http.fields={:?}", &fields);
            let mut form = multipart::Form::new();
            for (key, value) in fields {
                form = form.text(key, value);
            }

            // handle csrf
            if let Some(token) = csrf.as_ref() {
                form = form.text(token.name.clone(), token.value.clone());
            }

            builder = builder.multipart(form);

            // we already added the --http-body value as fields
            do_body = false;
        }

        // do we have any fields left to add?
        if do_body && self.payload.is_some() {
            if method_requires_payload(&self.method) {
                // check if we have to urlencode fields
                if self.headers.get(CONTENT_TYPE).unwrap() == "application/x-www-form-urlencoded" {
                    let mut form_fields =
                        payload::parse_fields(self.payload.as_ref(), creds).unwrap();

                    // handle csrf
                    if let Some(token) = csrf.as_ref() {
                        form_fields.push((token.name.to_owned(), token.value.to_owned()));
                    }

                    builder = builder.form(&form_fields);
                } else {
                    // add as raw body
                    let mut body = payload::parse_body(self.payload.as_ref(), creds).unwrap();
                    // handle csrf
                    if let Some(token) = csrf.as_ref() {
                        body.push_str(&format!("&{}={}", token.name, token.value));
                    }
                    builder = builder.body(body);
                }
            } else {
                // add as query string
                let mut query_fields = payload::parse_fields(self.payload.as_ref(), creds).unwrap();

                // handle csrf
                if let Some(token) = csrf.as_ref() {
                    query_fields.push((token.name.clone(), token.value.clone()));
                }

                // log::info!("http.query={:?}", &query);
                builder = builder.query(&query_fields);
            }
        }

        builder
    }

    async fn is_success_response(
        &self,
        creds: &Credentials,
        response: Response,
    ) -> Option<Success> {
        let status = response.status().as_u16();
        log::debug!("status={}", status);

        let content_type = if let Some(ctype) = response.headers().get(CONTENT_TYPE) {
            ctype.to_str().unwrap().split(';').collect::<Vec<&str>>()[0].to_owned()
        } else {
            String::new()
        };
        let headers = format!("{:?}", response.headers());
        let body = response.text().await.unwrap_or(String::new());
        let content_length = body.len();

        self.is_success(creds, status, content_type, content_length, headers, body)
            .await
    }

    async fn is_success(
        &self,
        creds: &Credentials,
        status: u16,
        content_type: String,
        content_length: usize,
        headers: String,
        body: String,
    ) -> Option<Success> {
        // check status first
        if !self.success_codes.contains(&status) {
            return None;
        }

        // if --http-success-string was provided, check for matches in the response
        let success_match = if let Some(success_string) = self.success_string.as_ref() {
            // perform interpolation
            let lookup = success_string
                .replace(HTTP_USERNAME_VAR, &creds.username)
                .replace(HTTP_PASSWORD_VAR, &creds.password)
                .replace(HTTP_PAYLOAD_VAR, creds.single());

            body.contains(&lookup) || headers.contains(&lookup)
        } else {
            true
        };

        let failure_match = if let Some(failure_string) = self.failure_string.as_ref() {
            // perform interpolation
            let lookup = failure_string
                .replace(HTTP_USERNAME_VAR, &creds.username)
                .replace(HTTP_PASSWORD_VAR, &creds.password)
                .replace(HTTP_PAYLOAD_VAR, creds.single());

            body.contains(&lookup) || headers.contains(&lookup)
        } else {
            false
        };

        if success_match && !failure_match {
            Some(Success {
                status,
                content_type,
                content_length,
                body,
            })
        } else {
            None
        }
    }

    fn setup_headers(&self) -> HeaderMap {
        let mut headers = self.headers.clone();

        let user_agent = if let Some(ua) = self.user_agent.as_ref() {
            // use selected user-agent
            ua.as_str()
        } else {
            // pick user-agent randomly
            ua::USER_AGENTS.choose(&mut rand::rng()).unwrap()
        };

        headers.append(USER_AGENT, HeaderValue::from_str(user_agent).unwrap());
        headers
    }

    async fn do_request(
        &self,
        opts: &Options,
        page: &str,
    ) -> Result<(Credentials, Response), Error> {
        let target = if let Some(real_target) = self.real_target.as_ref() {
            log::debug!("request with real target {}", real_target);
            real_target.to_owned()
        } else {
            log::debug!("request with opts.target {}", opts.target.as_ref().unwrap());
            opts.target.as_ref().unwrap().to_owned()
        };

        log::debug!("before: {}", target);
        let parsable = format!(
            "{}{}",
            if target.contains("://") {
                ""
            } else {
                "https://"
            },
            target
        );
        let parsed = Url::options()
            .parse(&parsable)
            .map_err(|e| format!("could not parse url '{}': {:?}", parsable, e))?;

        let target = if let Some(port) = parsed.port() {
            format!("{}://{}:{}", parsed.scheme(), parsed.host().unwrap(), port)
        } else {
            format!("{}://{}", parsed.scheme(), parsed.host().unwrap())
        };
        log::debug!("after: {}", &target);

        let url_raw = format!(
            "{}{}{}{}",
            if target.contains("://") {
                ""
            } else {
                "https://"
            },
            target,
            if target.ends_with('/') { "" } else { "/" },
            if page.starts_with('/') {
                page.strip_prefix('/').unwrap()
            } else {
                page
            }
        );

        log::debug!("  REQUEST TO {}", &url_raw);

        let url = Url::options()
            .parse(&url_raw)
            .map_err(|e| format!("could not parse url '{}': {:?}", url_raw, e))?;
        let headers = self.setup_headers();
        let request = self
            .client
            .request(self.method.clone(), url)
            .headers(headers);

        request.send().await.map_err(|e| e.to_string()).map(|res| {
            (
                Credentials {
                    target,
                    username: page.to_string(),
                    password: "".to_string(),
                },
                res,
            )
        })
    }

    async fn check_false_positives(&mut self, opts: &Options, adjust: bool) -> Result<(), Error> {
        let random_page = random_string::generate(5, "abcdefghijklmnopqrstuvwxyz");
        log::debug!(
            "check_false_positives: adjust={}, page={}",
            adjust,
            &random_page
        );
        if let Ok((creds, res)) = self.do_request(opts, &random_page).await {
            if let Some(success) = self.is_success_response(&creds, res).await {
                let target = if let Some(real_target) = self.real_target.as_ref() {
                    real_target.to_owned()
                } else {
                    opts.target.as_ref().unwrap().to_owned()
                };
                if adjust {
                    self.failure_string = Some(success.body);
                    return Err(format!(
                        "{} returned status code {} for a non existent page, adjusted --http-failure-string",
                        target, success.status
                    ));
                } else {
                    return Err(format!(
                        "{} returned again a status code {} for a non existent page, aborting due to likely false positives",
                        target, success.status
                    ));
                }
            }
        }

        Ok(())
    }

    async fn check_dot_false_positives(
        &mut self,
        opts: &Options,
        adjust: bool,
    ) -> Result<(), Error> {
        let random_page = format!(
            ".{}",
            random_string::generate(5, "abcdefghijklmnopqrstuvwxyz")
        );
        log::debug!(
            "check_dot_false_positives: adjust={}, page={}",
            adjust,
            &random_page
        );
        if let Ok((creds, res)) = self.do_request(opts, &random_page).await {
            if let Some(success) = self.is_success_response(&creds, res).await {
                let target = if let Some(real_target) = self.real_target.as_ref() {
                    real_target.to_owned()
                } else {
                    opts.target.as_ref().unwrap().to_owned()
                };
                if adjust {
                    // log::debug!("success={:?}", &success);
                    self.failure_string = Some(success.body);
                    return Err(format!(
                        "{} returned status code {} for a non existent page starting with a dot, adjusted --http-failure-string",
                        target, success.status
                    ));
                } else {
                    return Err(format!(
                        "{} returned again a status code {} for a non existent page starting with a dot, aborting due to likely false positives",
                        target, success.status
                    ));
                }
            }
        }

        Ok(())
    }

    async fn check_false_negatives(&mut self, opts: &Options) -> Result<(), Error> {
        let page = "/".to_string();
        log::debug!("check_false_negatives: page={}", &page);
        let result = self.do_request(opts, &page).await;
        if let Ok((creds, res)) = result {
            let status = res.status();
            let headers = res.headers().clone();
            if self.is_success_response(&creds, res).await.is_none() {
                let relocation = headers.get("location");
                if status.is_redirection() && relocation.is_some() {
                    let mut relocation = relocation.unwrap().to_str().unwrap().to_owned();
                    // redirect to a page
                    if relocation.starts_with("/") || relocation.contains(&creds.target) {
                        return Ok(());
                    } else if relocation.ends_with('/') {
                        relocation.pop();
                    }
                    self.real_target = Some(relocation.clone());
                    return Err(format!(
                        "{} returned status code {} for an existing page, adjusted to real target {}",
                        opts.target.as_ref().unwrap(),
                        status,
                        relocation
                    ));
                } else {
                    return Err(format!(
                        "{} returned status code {} for an existing page, aborting due to likely false negatives",
                        opts.target.as_ref().unwrap(),
                        status,
                    ));
                }
            }
        } else {
            return Err(format!(
                "{} returned an error for an existing page, aborting due to likely false negatives: {}",
                opts.target.as_ref().unwrap(),
                result.err().unwrap()
            ));
        }

        Ok(())
    }

    async fn check_status_codes(&mut self, opts: &Options) -> Result<(), Error> {
        if opts.target.is_none() {
            log::warn!("target not set, skipping status code check (TEST MODE?)");
            return Ok(());
        }

        log::info!("validating canary status codes ...");

        // check that the target is not returning 404 for an existing page
        // attempt this a few times since there might be multiple redirects
        // for instance:
        //    domain.com -> http://www.domain.com -> https://www.domain.com
        for _ in 0..10 {
            if let Err(e) = self.check_false_negatives(opts).await {
                log::warn!("{}", e);
            } else {
                break;
            }
        }

        // check that the target is not returning 404 for a non existent page starting with a dot
        if let Err(e) = self.check_dot_false_positives(opts, true).await {
            log::warn!("{}", e);
        }

        // check that the target is not returning 200 for a non existent page
        if let Err(e) = self.check_false_positives(opts, true).await {
            log::warn!("{}", e);
        }

        Ok(())
    }

    async fn http_request_attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let mut creds = creds.clone();
        let target = self.get_target_url(&mut creds)?;
        let mut headers = self.setup_headers();

        // check if we are in a ntlm auth challenge context
        if matches!(self.strategy, Strategy::NLTMv1 | Strategy::NLTMv2) {
            let auth = ntlm::handle(
                if self.strategy == Strategy::NLTMv1 {
                    1
                } else {
                    2
                },
                &target,
                self.client.clone(),
                &creds,
                &self.domain,
                &self.workstation,
                headers.clone(),
            )
            .await?;
            for (key, value) in auth.iter() {
                headers.append(key, value.clone());
            }
        }

        // check if we have to grab a CSRF token first
        let csrf_token = if let Some(csrf_config) = self.csrf.as_ref() {
            let token =
                csrf::handle(csrf_config, self.client.clone(), headers.clone(), timeout).await?;

            if let Some(token) = token.as_ref() {
                // set session cookie for CSRF
                if !token.cookie.is_empty() {
                    headers.append(COOKIE, HeaderValue::from_str(&token.cookie).unwrap());
                }
            }

            token
        } else {
            None
        };

        log::debug!("target={}", &target);

        // build base request object
        let mut request = self
            .client
            .request(self.method.clone(), &target)
            .headers(headers)
            .timeout(timeout);

        // setup body
        request = self.setup_request_body(&creds, csrf_token, request);
        // execute
        match request.send().await {
            Err(e) => Err(e.to_string()),
            Ok(res) => {
                let cookie = if let Some(cookie) = res.headers().get(COOKIE) {
                    cookie.to_str().unwrap().to_owned()
                } else {
                    "".to_owned()
                };
                Ok(if self.is_success_response(&creds, res).await.is_some() {
                    Some(vec![Loot::new(
                        "http",
                        &target,
                        [
                            ("username".to_owned(), creds.username.to_owned()),
                            ("password".to_owned(), creds.password.to_owned()),
                            ("cookie".to_owned(), cookie),
                        ],
                    )])
                } else {
                    None
                })
            }
        }
    }

    async fn http_enum_attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let mut creds = creds.clone();
        let target = self.get_target_url(&mut creds)?;
        let headers = self.setup_headers();
        let url_raw = if target.contains("{PAYLOAD}") {
            // by interpolation
            placeholders::interpolate(&target, &creds)
        } else {
            // by appending
            format!(
                "{}{}",
                &target,
                creds
                    .username
                    .replace(&self.enum_ext_placeholder, &self.enum_ext)
            )
        };

        // HACK: since crates.io removes the patch.crates-io sections from the Cargo file:
        //
        //  https://stackoverflow.com/questions/69235287/can-i-publish-a-crate-that-uses-a-patch
        //  https://github.com/rust-lang/cargo/issues/10440
        //
        // using our version of the URL crate won't compile with "cargo publish". Therefore
        // we need to wrap this in an optional feature that's not included by default.

        #[cfg(feature = "http_relative_paths")]
        let url = Url::options()
            .leave_relative(true)
            .parse(&url_raw)
            .map_err(|e| format!("could not parse url '{}': {:?}", url_raw, e))?;

        #[cfg(not(feature = "http_relative_paths"))]
        let url = Url::options()
            .parse(&url_raw)
            .map_err(|e| format!("could not parse url '{}': {:?}", url_raw, e))?;

        log::debug!("http.enum to {}", url_raw);

        // build base request object
        let request = self
            .client
            .request(self.method.clone(), url)
            .headers(headers)
            .timeout(timeout);

        // execute
        match request.send().await {
            Err(e) => Err(e.to_string()),
            Ok(res) => {
                if let Some(success) = self.is_success_response(&creds, res).await {
                    Ok(Some(vec![Loot::new(
                        "http.enum",
                        &target,
                        [
                            ("page".to_owned(), url_raw),
                            ("status".to_owned(), success.status.to_string()),
                            ("size".to_owned(), success.content_length.to_string()),
                            ("type".to_owned(), success.content_type),
                        ],
                    )]))
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn http_vhost_enum_attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        let mut creds = creds.clone();
        let url = self.get_target_url(&mut creds)?;
        let mut headers = self.setup_headers();

        // set host
        headers.remove(HOST);
        headers.insert(HOST, HeaderValue::from_str(&creds.username).unwrap());

        // build base request object
        let request = self
            .client
            .request(self.method.clone(), &url)
            .headers(headers)
            .timeout(timeout);

        log::debug!("http.vhost-enum to {} (host={})", url, &creds.username);

        // execute
        match request.send().await {
            Err(e) => Err(e.to_string()),
            Ok(res) => {
                if let Some(success) = self.is_success_response(&creds, res).await {
                    Ok(Some(vec![Loot::new(
                        "http.vhost",
                        &creds.target,
                        [
                            ("vhost".to_owned(), creds.username.to_owned()),
                            ("status".to_owned(), success.status.to_string()),
                            ("size".to_owned(), success.content_length.to_string()),
                            ("type".to_owned(), success.content_type),
                        ],
                    )]))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[async_trait]
impl Plugin for HTTP {
    fn description(&self) -> &'static str {
        match self.strategy {
            Strategy::Request => "HTTP request.",
            Strategy::Form => "HTTP multipart form request.",
            Strategy::BasicAuth => "HTTP basic authentication.",
            Strategy::NLTMv1 => "NTLMv1 authentication over HTTP.",
            Strategy::NLTMv2 => "NTLMv2 authentication over HTTP.",
            Strategy::Enumeration => "HTTP pages enumeration.",
            Strategy::VHostEnum => "HTTP virtual host enumeration.",
        }
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        match self.strategy {
            Strategy::Enumeration | Strategy::VHostEnum => PayloadStrategy::Single,
            _ => PayloadStrategy::UsernamePassword,
        }
    }

    async fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.user_agent = opts.http.http_ua.clone();

        self.csrf = if let Some(csrf_page) = opts.http.http_csrf_page.as_ref() {
            Some(csrf::Config::new(csrf_page, &opts.http.http_csrf_regexp)?)
        } else {
            None
        };

        if matches!(self.strategy, Strategy::NLTMv1 | Strategy::NLTMv2) {
            self.workstation = opts.http.http_ntlm_workstation.clone();
            if let Some(domain) = &opts.http.http_ntlm_domain {
                self.domain = domain.clone();
            } else {
                return Err("no --http-ntlm-domain specified".to_string());
            }
        }

        self.method =
            Method::from_bytes(opts.http.http_method.as_bytes()).map_err(|e| e.to_string())?;

        for keyvalue in &opts.http.http_headers {
            let parts: Vec<&str> = keyvalue.splitn(2, '=').collect();
            self.headers.insert(
                HeaderName::from_bytes(parts[0].as_bytes()).map_err(|e| e.to_string())?,
                HeaderValue::from_str(parts[1]).map_err(|e| e.to_string())?,
            );
        }

        if method_requires_payload(&self.method) {
            // check if Content-Type is set already, if not set default (tnx to @zip609)
            if !self.headers.contains_key("Content-Type") {
                self.headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                );
            }
        }

        self.payload = if let Some(payload) = &opts.http.http_payload {
            // check if we have a raw value or a file
            if let Some(filename) = payload.strip_prefix('@') {
                Some(
                    std::fs::read_to_string(filename)
                        .map_err(|e| format!("could not load {}: {}", filename, e))?,
                )
            } else {
                Some(payload.clone())
            }
        } else {
            None
        };

        self.success_string = opts.http.http_success_string.clone();
        self.failure_string = opts.http.http_failure_string.clone();
        self.success_codes = opts
            .http
            .http_success_codes
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<u16>().unwrap())
            .collect();

        self.enum_ext = opts.http.http_enum_ext.clone();
        self.enum_ext_placeholder = opts.http.http_enum_ext_placeholder.clone();

        if let Some(proxy) = &opts.http.proxy {
            self.proxy = Some(proxy.to_owned());
            if let Some(auth) = &opts.http.proxy_auth {
                let parts: Vec<&str> = auth.splitn(2, ':').collect();
                self.proxy_user = Some(parts[0].to_owned());
                self.proxy_pass = Some(parts[1].to_owned());
            }
        }

        // build the client
        let redirect_policy = if opts.http.http_follow_redirects {
            redirect::Policy::limited(255)
        } else {
            redirect::Policy::none()
        };

        self.client = if let Some(proxy) = &self.proxy {
            // add proxy if specified
            let mut proxy = reqwest::Proxy::all(proxy).map_err(|e| e.to_string())?;
            if self.proxy_user.is_some() && self.proxy_pass.is_some() {
                // set proxy authentication
                proxy = proxy.basic_auth(
                    self.proxy_user.as_ref().unwrap(),
                    self.proxy_pass.as_ref().unwrap(),
                );
            }

            reqwest::Client::builder()
                .proxy(proxy) // sets auto_sys_proxy to false, see https://github.com/evilsocket/legba/issues/8
                .danger_accept_invalid_certs(true)
                .redirect(redirect_policy)
                .build()
                .map_err(|e| e.to_string())?
        } else {
            // plain client
            reqwest::Client::builder()
                .no_proxy() // used to set auto_sys_proxy to false, see https://github.com/evilsocket/legba/issues/8
                .danger_accept_invalid_certs(true)
                .redirect(redirect_policy)
                .build()
                .map_err(|e| e.to_string())?
        };

        self.check_status_codes(opts).await
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        match self.strategy {
            Strategy::Enumeration => self.http_enum_attempt(creds, timeout).await,
            Strategy::VHostEnum => self.http_vhost_enum_attempt(creds, timeout).await,
            _ => self.http_request_attempt(creds, timeout).await,
        }
    }
}

// TODO: add more tests
#[cfg(test)]
mod tests {
    use reqwest::header::{CONTENT_TYPE, HeaderValue};

    use crate::{
        creds::Credentials,
        options::Options,
        plugins::{
            Plugin,
            http::{HTTP_PASSWORD_VAR, HTTP_PAYLOAD_VAR, HTTP_USERNAME_VAR},
        },
    };

    use super::{HTTP, Strategy};

    #[test]
    fn test_get_target_url_adds_default_schema_and_path() {
        let mut creds = Credentials {
            target: "localhost:3000".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_adds_default_schema() {
        let mut creds = Credentials {
            target: "localhost:3000/somepath".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/somepath",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_adds_default_path() {
        let mut creds = Credentials {
            target: "https://localhost:3000".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_preserves_query() {
        let mut creds = Credentials {
            target: "localhost:3000/?foo=bar".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?foo=bar",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_with_username_placeholder() {
        let mut creds = Credentials {
            target: "localhost:3000/?username={USERNAME}".to_owned(),
            username: "bob".to_owned(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?username=bob",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_with_password_placeholder() {
        let mut creds = Credentials {
            target: "localhost:3000/?p={PASSWORD}".to_owned(),
            username: String::new(),
            password: "f00b4r".to_owned(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?p=f00b4r",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_with_payload_placeholder() {
        let mut creds = Credentials {
            target: "localhost:3000/?p={PAYLOAD}".to_owned(),
            username: "something".to_owned(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?p=something",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_urlencoded() {
        let mut creds = Credentials {
            target: "localhost:3000/?p=%7BPAYLOAD%7D".to_owned(),
            username: "something".to_owned(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?p=something",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[tokio::test]
    async fn test_plugin_adds_default_content_type_if_post() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();

        opts.http.http_method = "POST".to_owned();
        opts.http.http_payload = Some("just a test".to_owned());

        assert_eq!(Ok(()), http.setup(&opts).await);
        assert_eq!(
            Some(&HeaderValue::from_static(
                "application/x-www-form-urlencoded"
            )),
            http.headers.get(CONTENT_TYPE)
        );
    }

    #[tokio::test]
    async fn test_plugin_preserves_user_content_type() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();

        opts.http.http_method = "POST".to_owned();
        opts.http.http_payload = Some("{\"foo\": 123}".to_owned());
        opts.http.http_headers = vec!["Content-Type=application/json".to_owned()];

        assert_eq!(Ok(()), http.setup(&opts).await);
        assert_eq!(
            Some(&HeaderValue::from_static("application/json")),
            http.headers.get(CONTENT_TYPE)
        );
    }

    #[tokio::test]
    async fn test_is_success() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = String::new();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_not_success() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some("login ok".to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "nope".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_is_success_match() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some("login ok".to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "sir login ok sir".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_http_enumeration_with_cyrillic_chars() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some("успех".to_owned());
        opts.http.http_method = "GET".to_owned();

        let mut creds = Credentials {
            target: "localhost:3000/тест/страница".to_owned(),
            username: "пользователь".to_owned(),
            password: "пароль".to_owned(),
        };

        let target_url = http.get_target_url(&mut creds).unwrap();
        assert_eq!(
            target_url,
            "https://localhost:3000/%D1%82%D0%B5%D1%81%D1%82/%D1%81%D1%82%D1%80%D0%B0%D0%BD%D0%B8%D1%86%D0%B0"
        );

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "операция успех завершена".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);
        assert_eq!(http.success_codes, vec![200]);
        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_success_custom_code() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "666".to_owned();
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 666;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "sir login ok sir".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![666]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_not_success_custom_code() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "666".to_owned();
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "sir login ok sir".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![666]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_is_success_with_negative_match() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_failure_string = Some("wrong credentials".to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "all good".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_not_success_with_negative_match() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_failure_string = Some("wrong credentials".to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "you sent the wrong credentials, freaking moron!".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_is_not_success_with_positive_and_negative_match() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some("credentials".to_owned());
        opts.http.http_failure_string = Some("wrong credentials".to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "you sent the wrong credentials, freaking moron!".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_is_success_with_positive_and_negative_match() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some("credentials".to_owned());
        opts.http.http_failure_string = Some("wrong credentials".to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials::default();

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "i like your credentials".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_success_with_interpolated_username() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some(HTTP_USERNAME_VAR.to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials {
            target: String::new(),
            username: "foo".to_owned(),
            password: String::new(),
        };

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "hello foo how are you doing?".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_success_with_interpolated_password() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some(HTTP_PASSWORD_VAR.to_owned());
        opts.http.http_method = "GET".to_owned();

        let creds = Credentials {
            target: String::new(),
            username: "foo".to_owned(),
            password: "p4ssw0rd".to_owned(),
        };

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "very cool p4ssw0rd buddy!".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_is_success_with_interpolated_payload() {
        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        "200".clone_into(&mut opts.http.http_success_codes);
        opts.http.http_success_string = Some(HTTP_PAYLOAD_VAR.to_owned());
        "GET".clone_into(&mut opts.http.http_method);

        let creds = Credentials {
            target: String::new(),
            username: "<svg onload=alert(1)>".to_owned(),
            password: String::new(),
        };

        let status = 200;
        let content_type = String::new();
        let content_length = 0;
        let headers = String::new();
        let body = "totally not vulnerable <svg onload=alert(1)> to xss".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        assert_eq!(http.success_codes, vec![200]);

        assert!(
            http.is_success(&creds, status, content_type, content_length, headers, body)
                .await
                .is_some()
        );
    }

    // Tests for check_status_codes and related methods

    #[tokio::test]
    async fn test_check_status_codes_skips_when_no_target() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        // opts.target is None by default

        opts.http.http_method = "GET".to_owned();

        // This should succeed because check_status_codes skips when target is None
        assert_eq!(Ok(()), http.setup(&opts).await);

        // Verify that the HTTP client was still set up correctly
        assert_eq!(http.method, reqwest::Method::GET);
    }

    #[tokio::test]
    async fn test_check_status_codes_normal_behavior() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();

        // 200 for /, 404 for any other path
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Home page</body></html>");
        });
        let _random_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });
        let _dot_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/\\.[a-z0-9]+$").unwrap());
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        opts.target = Some(server.base_url());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success_codes = "200".to_owned();
        let result = http.setup(&opts).await;
        assert_eq!(Ok(()), result);
        // Don't assert mocks, just ensure no panic
    }

    #[tokio::test]
    async fn test_check_false_positives_adjusts_failure_string() {
        use httpmock::prelude::*;
        use regex::Regex;
        let server = MockServer::start();
        // 200 for /, 200 for random paths
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Home page</body></html>");
        });
        let _random_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Success page</body></html>");
        });
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        opts.target = Some(server.base_url());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success_codes = "200".to_owned();
        let result = http.setup(&opts).await;
        assert_eq!(result, Ok(()));
        assert_eq!(
            http.failure_string,
            Some("<html><body>Success page</body></html>".to_owned())
        );
    }

    #[tokio::test]
    async fn test_check_dot_false_positives_adjusts_failure_string() {
        use httpmock::prelude::*;
        use regex::Regex;
        let server = MockServer::start();
        // 200 for /, 200 for dot pages, 200 for other random paths
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Home page</body></html>");
        });
        let _dot_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/\.[a-z0-9]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Dot page success</body></html>");
        });
        let _other_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Success page</body></html>");
        });
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        opts.target = Some(server.base_url());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success_codes = "200".to_owned();
        let result = http.setup(&opts).await;
        assert_eq!(result, Ok(()));
        let failure_string = http.failure_string.clone();
        assert!(
            failure_string == Some("<html><body>Dot page success</body></html>".to_owned())
                || failure_string == Some("<html><body>Success page</body></html>".to_owned())
        );
    }

    #[tokio::test]
    async fn test_check_false_negatives_handles_redirects() {
        use httpmock::prelude::*;
        let server = MockServer::start();
        // 302 for /, 404 for others
        let _redirect_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(302)
                .header("Location", "https://completely.different.domain/")
                .header("content-type", "text/html")
                .body("<html><body>Redirecting...</body></html>");
        });
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        opts.target = Some(server.base_url());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success_codes = "200".to_owned();
        let result = http.setup(&opts).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("aborting due to likely false negatives")
        );
        assert_eq!(
            http.real_target,
            Some("https://completely.different.domain".to_owned())
        );
    }

    #[tokio::test]
    async fn test_check_status_codes_with_success_and_failure_strings() {
        use httpmock::prelude::*;
        use regex::Regex;
        let server = MockServer::start();
        // 200 for /, 200 for random with failure content
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Home page</body></html>");
        });
        let _failure_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("This is a failure page with specific content and the word failure");
        });
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        opts.target = Some(server.base_url());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success_codes = "200".to_owned();
        opts.http.http_success_string = Some("success".to_owned());
        opts.http.http_failure_string = Some("failure".to_owned());
        let result = http.setup(&opts).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("adjusted --http-failure-string")
                || err.contains("aborting due to likely false positives")
                || err.contains("aborting due to likely false negatives")
        );
        assert_eq!(http.success_string, Some("success".to_owned()));
        let failure_string = http.failure_string.clone();
        assert!(
            failure_string == Some("failure".to_owned())
                || failure_string
                    == Some(
                        "This is a failure page with specific content and the word failure"
                            .to_owned()
                    )
        );
    }

    #[tokio::test]
    async fn test_check_status_codes_different_strategies() {
        use httpmock::prelude::*;
        use regex::Regex;
        let strategies = vec![
            Strategy::Request,
            Strategy::Form,
            Strategy::BasicAuth,
            Strategy::Enumeration,
            Strategy::VHostEnum,
        ];
        for strategy in strategies {
            let server = MockServer::start();
            let _home_mock = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(200)
                    .header("content-type", "text/html")
                    .body("<html><body>Home page</body></html>");
            });
            let _random_mock = server.mock(|when, then| {
                when.method(GET)
                    .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
                then.status(404)
                    .header("content-type", "text/html")
                    .body("<html><body>Not found</body></html>");
            });
            let mut http = HTTP::new(strategy);
            let mut opts = Options::default();
            opts.target = Some(server.base_url());
            opts.http.http_method = "GET".to_owned();
            opts.http.http_success_codes = "200".to_owned();
            let result = http.setup(&opts).await;
            assert_eq!(Ok(()), result);
        }
    }

    #[tokio::test]
    async fn test_check_status_codes_error_handling() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();

        opts.target = Some("http://invalid-host-that-will-fail:9999".to_owned());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success_codes = "200".to_owned();

        // This should fail due to connection error
        let result = http.setup(&opts).await;
        assert!(result.is_err());
    }
}
