use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use rand::seq::SliceRandom;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart, redirect, Client, Method, RequestBuilder, Response,
};
use url::Url;

use crate::session::{Error, Loot};
use crate::Options;

use crate::creds::Credentials;
use crate::plugins::Plugin;

mod csrf;
mod ntlm;
pub(crate) mod options;
mod payload;
mod ua;

#[ctor]
fn register() {
    crate::plugins::manager::register("http", Box::new(HTTP::new(Strategy::Request)));
    crate::plugins::manager::register("http.form", Box::new(HTTP::new(Strategy::Form)));
    crate::plugins::manager::register("http.basic", Box::new(HTTP::new(Strategy::BasicAuth)));
    crate::plugins::manager::register("http.ntlm1", Box::new(HTTP::new(Strategy::NLTMv1)));
    crate::plugins::manager::register("http.ntlm2", Box::new(HTTP::new(Strategy::NLTMv2)));
    crate::plugins::manager::register("http.enum", Box::new(HTTP::new(Strategy::Enumeration)));
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
}

struct Success {
    pub status: u16,
    pub content_type: String,
    pub content_length: usize,
}

#[derive(Clone)]
pub(crate) struct HTTP {
    strategy: Strategy,
    client: Client,

    target: String,

    csrf: Option<csrf::Config>,

    domain: String,
    workstation: String,

    random_ua: bool,
    success_codes: Vec<u16>,
    success_string: Option<String>,

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
            client: Client::default(),
            target: String::new(),
            csrf: None,
            domain: String::new(),
            workstation: String::new(),
            success_codes: vec![200],
            success_string: None,
            enum_ext: String::new(),
            enum_ext_placeholder: String::new(),
            method: Method::GET,
            headers: HeaderMap::default(),
            random_ua: false,
            payload: None,
            proxy: None,
            proxy_user: None,
            proxy_pass: None,
        }
    }

    fn setup_request_body(
        &self,
        creds: &Credentials,
        csrf: Option<csrf::Token>,
        mut request: RequestBuilder,
    ) -> RequestBuilder {
        let mut do_body = true;
        if self.strategy == Strategy::BasicAuth {
            // set basic authentication data
            request = request.basic_auth(&creds.username, Some(&creds.password));
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

            request = request.multipart(form);

            // we already added the --http-body value as fields
            do_body = false;
        }

        // do we have any fields left to add?
        if do_body && self.payload.is_some() {
            if method_requires_payload(&self.method) {
                // add as body
                let mut body = payload::parse_body(self.payload.as_ref(), creds).unwrap();

                // handle csrf
                if let Some(token) = csrf.as_ref() {
                    body.push_str(&format!("&{}={}", token.name, token.value));
                }

                // log::info!("http.body={}", &body);
                request = request
                    .body(body)
                    .header("Content-Type", "application/x-www-form-urlencoded");
            } else {
                // add as query string
                let mut query = payload::parse_fields(self.payload.as_ref(), creds).unwrap();

                // handle csrf
                if let Some(token) = csrf.as_ref() {
                    query.push((token.name.clone(), token.value.clone()));
                }

                // log::info!("http.query={:?}", &query);
                request = request.query(&query);
            }
        }

        request
    }

    async fn is_success(&self, response: Response) -> Option<Success> {
        let status = response.status().as_u16();
        if !self.success_codes.contains(&status) {
            return None;
        }

        let content_type = if let Some(ctype) = response.headers().get("content-type") {
            ctype
                .to_str()
                .unwrap()
                .to_owned()
                .split(';')
                .collect::<Vec<&str>>()[0]
                .to_owned()
        } else {
            String::new()
        };
        let headers = format!("{:?}", response.headers());
        let body = response.text().await.unwrap_or(String::new());
        let content_length = body.len();

        if let Some(success_string) = self.success_string.as_ref() {
            if !body.contains(success_string) && !headers.contains(success_string) {
                return None;
            }
        }

        Some(Success {
            status,
            content_type,
            content_length,
        })
    }

    fn setup_headers(&self) -> HeaderMap {
        let mut headers = self.headers.clone();

        if self.random_ua {
            headers.append(
                "User-Agent",
                HeaderValue::from_str(ua::USER_AGENTS.choose(&mut rand::thread_rng()).unwrap())
                    .unwrap(),
            );
        }

        headers
    }

    async fn http_request_attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Loot>, Error> {
        let mut headers = self.setup_headers();

        // check if we are in a ntlm auth challenge context
        if matches!(self.strategy, Strategy::NLTMv1 | Strategy::NLTMv2) {
            let auth = ntlm::handle(
                if self.strategy == Strategy::NLTMv1 {
                    1
                } else {
                    2
                },
                &self.target,
                self.client.clone(),
                creds,
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
                    headers.append("Cookie", HeaderValue::from_str(&token.cookie).unwrap());
                }
            }

            token
        } else {
            None
        };

        // build base request object
        let mut request = self
            .client
            .request(self.method.clone(), &self.target)
            .headers(headers)
            .timeout(timeout);

        // setup body
        request = self.setup_request_body(creds, csrf_token, request);

        // execute
        match request.send().await {
            Err(e) => Err(e.to_string()),
            Ok(res) => {
                let cookie = if let Some(cookie) = res.headers().get("cookie") {
                    cookie.to_str().unwrap().to_owned()
                } else {
                    "".to_owned()
                };
                Ok(if self.is_success(res).await.is_some() {
                    Some(Loot::from(
                        &self.target,
                        [
                            ("username".to_owned(), creds.username.to_owned()),
                            ("password".to_owned(), creds.password.to_owned()),
                            ("cookie".to_owned(), cookie),
                        ],
                    ))
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
    ) -> Result<Option<Loot>, Error> {
        let headers = self.setup_headers();

        let url = if self.target.contains("{PAYLOAD}") {
            // by interpolation
            self.target.replace("{PAYLOAD}", &creds.username)
        } else {
            // by appending
            format!(
                "{}{}",
                &self.target,
                creds
                    .username
                    .replace(&self.enum_ext_placeholder, &self.enum_ext)
            )
        };

        // build base request object
        let request = self
            .client
            .request(self.method.clone(), &url)
            .headers(headers)
            .timeout(timeout);

        // execute
        match request.send().await {
            Err(e) => Err(e.to_string()),
            Ok(res) => {
                if let Some(success) = self.is_success(res).await {
                    Ok(Some(Loot::from(
                        &self.target,
                        [
                            ("page".to_owned(), url),
                            ("status".to_owned(), success.status.to_string()),
                            ("size".to_owned(), success.content_length.to_string()),
                            ("type".to_owned(), success.content_type),
                        ],
                    )))
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
        }
    }

    fn single_credential(&self) -> bool {
        self.strategy == Strategy::Enumeration
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        if let Some(target) = opts.target.as_ref() {
            // add default schema if not present
            let target = if !target.contains("://") {
                format!("http://{}", target)
            } else {
                target.to_owned()
            };

            // parse as url
            let target_url = Url::parse(&target).map_err(|e| e.to_string())?;
            self.target = if self.strategy == Strategy::Enumeration {
                let port_part = if let Some(port) = target_url.port() {
                    format!(":{}", port)
                } else {
                    "".to_owned()
                };

                let path = target_url
                    .path()
                    .replace("%7BUSERNAME%7D", "{USERNAME}")
                    .replace("%7BPASSWORD%7D", "{PASSWORD}")
                    .replace("%7BPAYLOAD%7D", "{PAYLOAD}"); // undo query encoding of interpolation params

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
        } else {
            return Err("no --target url specified".to_string());
        }

        self.random_ua = opts.http.http_random_ua;

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

        // check that a payload was provided if needed
        if method_requires_payload(&self.method) && opts.http.http_payload.is_none() {
            return Err(format!(
                "method {} requires an --http-payload value",
                self.method
            ));
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
                .proxy(proxy)
                .danger_accept_invalid_certs(true)
                .redirect(redirect_policy)
                .build()
                .map_err(|e| e.to_string())?
        } else {
            // plain client
            reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .redirect(redirect_policy)
                .build()
                .map_err(|e| e.to_string())?
        };

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        if self.strategy == Strategy::Enumeration {
            self.http_enum_attempt(creds, timeout).await
        } else {
            self.http_request_attempt(creds, timeout).await
        }
    }
}
