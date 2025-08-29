use std::time::Duration;

use async_trait::async_trait;
use evalexpr::*;
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

const HTTP_LOWERCASE_PLACEHOLDERS: &[&str] = &["{payload}", "{username}", "{password}"];
const HTTP_UPPERCASE_PLACEHOLDERS: &[&str] = &["{PAYLOAD}", "{USERNAME}", "{PASSWORD}"];

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
    pub size: usize,
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

    success_expression: evalexpr::Node<evalexpr::DefaultNumericTypes>,

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
            enum_ext: String::new(),
            success_expression: evalexpr::build_operator_tree("").unwrap(),
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
        let mut target_url = if self.strategy == Strategy::Enumeration {
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

        // the Url::parse() call in get_target_url will make the placeholders lowercase
        // if they are in the hostname, we need to re-uppercase them in order for the
        // next part of the logic to work
        for placeholder in HTTP_LOWERCASE_PLACEHOLDERS {
            target_url = target_url.replace(placeholder, &placeholder.to_uppercase());
        }

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

    async fn build_response_context(
        &self,
        creds: &Credentials,
        response: Response,
    ) -> Result<(u16, String, usize, HashMapContext<DefaultNumericTypes>), Error> {
        let status = response.status().as_u16();

        let mut context = HashMapContext::<DefaultNumericTypes>::new();
        let mut content_type_set = false;
        let mut content_type = String::new();

        // add headers to the context
        for (key, value) in response.headers().iter() {
            let header_var_name = key.to_string().to_lowercase().replace("-", "_");
            #[cfg(test)]
            println!(
                "adding header '{}' to context as '{}'",
                key, header_var_name
            );

            if header_var_name == "content_type" {
                content_type_set = true;
                content_type = value.to_str().unwrap().to_owned();
            }

            context
                .set_value(header_var_name, Value::from(value.to_str().unwrap()))
                .map_err(|e| e.to_string())?;
        }

        // always set content_type
        if !content_type_set {
            context
                .set_value(String::from("content_type"), Value::from(""))
                .map_err(|e| e.to_string())?;
        }

        // add response status, body, size and content type to the context
        let body = response.text().await.unwrap_or(String::new());
        let size = body.len();

        context
            .set_value(String::from("status"), Value::from_int(status as i64))
            .map_err(|e| e.to_string())?;
        context
            .set_value(String::from("body"), Value::from(body))
            .map_err(|e| e.to_string())?;
        context
            .set_value(String::from("size"), Value::from_int(size as i64))
            .map_err(|e| e.to_string())?;

        // the builtin contains function is for searching a string within a tuple of strings,
        // let's override it to something that makes more sense
        context
            .set_function(
                String::from("contains"),
                Function::new(|argument| {
                    let arguments = argument.as_fixed_len_tuple(2)?;
                    if let (Value::String(a), Value::String(b)) =
                        (&arguments[0].clone(), &arguments[1].clone())
                    {
                        Ok(Value::from(a.contains(b)))
                    } else {
                        Err(EvalexprError::expected_tuple(arguments[0].clone()))
                    }
                }),
            )
            .map_err(|e| e.to_string())?;

        // set placeholders with actual values
        context
            .set_value(
                String::from("username"),
                Value::from(creds.username.clone()),
            )
            .map_err(|e| e.to_string())?;
        context
            .set_value(
                String::from("password"),
                Value::from(creds.password.clone()),
            )
            .map_err(|e| e.to_string())?;
        context
            .set_value(String::from("payload"), Value::from(creds.single()))
            .map_err(|e| e.to_string())?;

        Ok((status, content_type, size, context))
    }

    async fn is_success_response(
        &self,
        creds: &Credentials,
        response: Response,
    ) -> Option<Success> {
        let built = self.build_response_context(creds, response).await;
        if let Ok((status, content_type, size, context)) = built {
            match self.success_expression.eval_boolean_with_context(&context) {
                Ok(result) => {
                    if result {
                        Some(Success {
                            status,
                            content_type,
                            size,
                        })
                    } else {
                        None
                    }
                }
                Err(e) => {
                    #[cfg(test)]
                    println!(
                        "error evaluating expression '{}': {}",
                        self.success_expression, e
                    );
                    #[cfg(not(test))]
                    log::error!("error evaluating success expression: {}", e);
                    None
                }
            }
        } else {
            log::error!("error building response context: {}", built.err().unwrap());
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

        match request.send().await {
            Ok(res) => Ok((
                Credentials {
                    target,
                    username: page.to_string(),
                    password: "".to_string(),
                },
                res,
            )),
            Err(e) => {
                // some errors are not entirely shown when using e.to_string(), for instance:
                // hyper_util::client::legacy::Error(Connect, Error { code: -9836, message: "bad protocol version" }) })
                // just becomes "error sending request"
                Err(format!("{:?}", e))
            }
        }
    }

    async fn check_false_positives(&mut self, opts: &Options, adjust: bool) -> Result<(), Error> {
        let random_page = random_string::generate(5, "abcdefghijklmnopqrstuvwxyz");
        log::debug!(
            "check_false_positives: adjust={}, page={}",
            adjust,
            &random_page
        );
        if let Ok((creds, res)) = self.do_request(opts, &random_page).await
            && let Some(success) = self.is_success_response(&creds, res).await
        {
            let target = if let Some(real_target) = self.real_target.as_ref() {
                real_target.to_owned()
            } else {
                opts.target.as_ref().unwrap().to_owned()
            };
            if adjust {
                return Err(format!(
                    "{} validates success condition for a non existent page, likely false positives: {:?}",
                    target, success
                ));
            } else {
                return Err(format!(
                    "aborting due to likely false positives for {}: validates success condition for a non existent page: {:?}",
                    target, success
                ));
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
        if let Ok((creds, res)) = self.do_request(opts, &random_page).await
            && let Some(success) = self.is_success_response(&creds, res).await
        {
            let target = if let Some(real_target) = self.real_target.as_ref() {
                real_target.to_owned()
            } else {
                opts.target.as_ref().unwrap().to_owned()
            };
            if adjust {
                // log::debug!("success={:?}", &success);
                return Err(format!(
                    "{} validates success condition for a non existent page starting with a dot, likely false positives: {:?}",
                    target, success
                ));
            } else {
                return Err(format!(
                    "aborting due to likely false positives for {}: validates success condition for a non existent page starting with a dot: {:?}",
                    target, success
                ));
            }
        }

        Ok(())
    }

    async fn check_false_negatives(&mut self, opts: &Options) -> Result<(), Error> {
        let page = "/".to_string();
        log::debug!("check_false_negatives: page={}", &page);
        let result = self.do_request(opts, &page).await;
        log::debug!("result: {:?}", result);
        if let Ok((creds, res)) = result {
            let status = res.status();
            let headers = res.headers().clone();
            if self.is_success_response(&creds, res).await.is_none() {
                let relocation = headers.get("location");
                if status.is_redirection()
                    && let Some(relocation) = relocation
                {
                    let mut relocation = relocation.to_str().unwrap().to_owned();
                    // redirect to a page
                    if relocation.starts_with("/") || relocation.contains(&creds.target) {
                        return Ok(());
                    } else if relocation.ends_with('/') {
                        relocation.pop();
                    }
                    self.real_target = Some(relocation.clone());
                    return Err(format!(
                        "{} returned a non success response for an existing page, adjusted to real target {}",
                        opts.target.as_ref().unwrap(),
                        relocation
                    ));
                } else {
                    return Err(
                        "success condition did not validate for for an existing page, likely false negatives".into()
                    );
                }
            }
        } else {
            return Err(format!(
                "{} returned an error for an existing page, likely false negatives: {}",
                opts.target.as_ref().unwrap(),
                result.err().unwrap()
            ));
        }

        Ok(())
    }

    async fn validate_success_condition(&mut self, opts: &Options) -> Result<(), Error> {
        if opts.target.is_none() {
            log::warn!("target not set, skipping status code check (TEST MODE?)");
            return Ok(());
        }

        let t = opts.target.as_ref().unwrap();
        for placeholder in HTTP_UPPERCASE_PLACEHOLDERS {
            if t.contains(placeholder) {
                log::info!("target contains a placeholder, skipping success condition check");
                return Ok(());
            }
        }

        log::info!(
            "validating http success condition: {}",
            self.success_expression
        );

        // check that the target is not returning 404 for an existing page
        // attempt this a few times since there might be multiple redirects
        // for instance:
        //    domain.com -> http://www.domain.com -> https://www.domain.com
        for _ in 0..5 {
            if let Err(e) = self.check_false_negatives(opts).await {
                log::warn!("{}", e);
            } else {
                break;
            }

            // if we are following redirects, we can stop
            if opts.http.http_follow_redirects {
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
        let mut had_placeholder = false;
        for placeholder in HTTP_UPPERCASE_PLACEHOLDERS {
            if creds.target.contains(placeholder) {
                had_placeholder = true;
                break;
            }
        }

        let target = self.get_target_url(&mut creds)?;
        let headers = self.setup_headers();

        // if the target itself contained a placeholder, the payload
        // has already been interpolated by get_target_url, so we don't need to do it again
        let url_raw = if had_placeholder {
            target.to_owned()
        } else {
            // otherwise, we need to append it
            format!(
                "{}{}",
                &target,
                creds
                    .username
                    .replace(&self.enum_ext_placeholder, &self.enum_ext)
            )
        };

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
                            ("size".to_owned(), success.size.to_string()),
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
                            ("size".to_owned(), success.size.to_string()),
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

        self.success_expression =
            evalexpr::build_operator_tree(&opts.http.http_success).map_err(|e| {
                format!(
                    "error parsing success expression '{}': {}",
                    opts.http.http_success, e
                )
            })?;

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
                // https://github.com/seanmonstar/reqwest/discussions/2428
                .use_rustls_tls()
                .redirect(redirect_policy)
                .build()
                .map_err(|e| e.to_string())?
        } else {
            // plain client
            reqwest::Client::builder()
                .no_proxy() // used to set auto_sys_proxy to false, see https://github.com/evilsocket/legba/issues/8
                .danger_accept_invalid_certs(true)
                // https://github.com/seanmonstar/reqwest/discussions/2428
                .use_rustls_tls()
                .redirect(redirect_policy)
                .build()
                .map_err(|e| e.to_string())?
        };

        self.validate_success_condition(opts).await
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

#[cfg(test)]
#[path = "http_test.rs"]
mod http_test;
