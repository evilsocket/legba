use std::time::Duration;

use crate::{
    plugins::port_scanner::options,
    utils::net::{StreamLike, upgrade_tcp_stream_to_tls},
};
use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;
use x509_parser::prelude::{FromDer, GeneralName, X509Certificate};

use super::Banner;

static HTML_TITLE_PARSER: Lazy<Regex> = lazy_regex!(r"(?i)<\s*title\s*>([^<]+)<\s*/\s*title\s*>");

pub(crate) fn is_http_port(opts: &options::Options, port: u16) -> (bool, bool) {
    if opts.port_scanner_http == "*" {
        return (true, false);
    }

    if opts.port_scanner_https == "*" {
        return (true, true);
    }

    for http_port in opts
        .port_scanner_http
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if port == http_port.parse::<u16>().unwrap() {
            return (true, false);
        }
    }

    for https_port in opts
        .port_scanner_https
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if port == https_port.parse::<u16>().unwrap() {
            return (true, true);
        }
    }

    (false, false)
}

pub(crate) async fn parse_http_response(
    opts: &options::Options,
    response: reqwest::Response,
    banner: &mut Banner,
    address: &str,
    port: u16,
) {
    let headers_of_interest: Vec<&str> = opts
        .port_scanner_http_headers
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let mut content_type = String::from("text/html");

    // collect headers
    for (name, value) in response.headers() {
        let name = name.to_string();
        let mut value = value.to_str().unwrap();

        if name == "content-type" {
            if value.contains(';') {
                value = value.split(';').next().unwrap();
            }
            value.clone_into(&mut content_type);
        }

        if headers_of_interest.contains(&name.as_str()) {
            banner.insert(name, value.to_owned());
        }
    }

    // collect info from html
    let body = response.text().await;
    if let Ok(body) = body {
        if content_type.contains("text/html") {
            if let Some(caps) = HTML_TITLE_PARSER.captures(&body) {
                banner.insert(
                    "html.title".to_owned(),
                    caps.get(1).unwrap().as_str().trim().to_owned(),
                );
            }
        } else if content_type.contains("application/") || content_type.contains("text/") {
            banner.insert("body".to_owned(), body.to_owned());
        }
    } else {
        log::debug!(
            "can't read response body from {}:{}: {:?}",
            address,
            port,
            body.err()
        );
    }
}

pub(crate) async fn parse_http_raw_response(
    opts: &options::Options,
    response: &str,
    banner: &mut Banner,
) {
    let headers_of_interest: Vec<&str> = opts
        .port_scanner_http_headers
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let mut content_type = String::from("text/html");

    // split response into headers and body
    let lines = response.lines();
    let mut headers_section = true;
    let mut body_lines = Vec::new();

    for line in lines {
        if headers_section {
            if line.trim().is_empty() {
                // empty line indicates end of headers
                headers_section = false;
            } else if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim();

                if name == "content-type" {
                    let mut ct_value = value;
                    if ct_value.contains(';') {
                        ct_value = ct_value.split(';').next().unwrap();
                    }
                    ct_value.clone_into(&mut content_type);
                }

                if headers_of_interest.contains(&name.as_str()) {
                    banner.insert(name, value.to_owned());
                }
            }
        } else {
            body_lines.push(line);
        }
    }

    // process body
    if !body_lines.is_empty() {
        let body = body_lines.join("\n");
        if content_type.contains("text/html") {
            if let Some(caps) = HTML_TITLE_PARSER.captures(&body) {
                banner.insert(
                    "html.title".to_owned(),
                    caps.get(1).unwrap().as_str().trim().to_owned(),
                );
            }
        } else if content_type.contains("application/") || content_type.contains("text/") {
            banner.insert("body".to_owned(), body);
        }
    }
}

pub(crate) async fn http_grabber(
    opts: &options::Options,
    host: &str,
    address: &str,
    port: u16,
    stream: Box<dyn StreamLike>,
    ssl: bool,
    timeout: Duration,
) -> Banner {
    let mut banner = Banner::default();

    banner.insert(
        "protocol".to_owned(),
        if ssl {
            "https".to_owned()
        } else {
            "http".to_owned()
        },
    );

    let url = format!(
        "{}://{}:{}/",
        if ssl { "https" } else { "http" },
        address,
        port
    );

    // if ssl, upgrade stream to get certificate information
    if ssl {
        let upgraded = upgrade_tcp_stream_to_tls(stream, host, timeout).await;
        if let Ok(tls) = upgraded {
            if let Ok(Some(cert)) = tls.peer_certificate() {
                if let Ok(der) = cert.to_der() {
                    if let Ok((_, cert)) = X509Certificate::from_der(&der) {
                        banner.insert("certificate.serial".to_owned(), cert.raw_serial_as_string());
                        banner.insert("certificate.subject".to_owned(), cert.subject().to_string());
                        banner.insert("certificate.issuer".to_owned(), cert.issuer().to_string());

                        let validity = cert.validity();
                        banner.insert(
                            "certificate.valid_from".to_owned(),
                            validity.not_before.to_string(),
                        );
                        banner.insert(
                            "certificate.valid_to".to_owned(),
                            validity.not_after.to_string(),
                        );

                        if let Ok(Some(alt_names)) = cert.subject_alternative_name() {
                            banner.insert(
                                "certificate.names".to_owned(),
                                alt_names
                                    .value
                                    .general_names
                                    .iter()
                                    .map(|n| match n {
                                        GeneralName::DNSName(s) => s.to_string(),
                                        _ => n.to_string(),
                                    })
                                    .collect::<Vec<String>>()
                                    .join(", "),
                            );
                        }
                    } else {
                        log::error!("failed to parse certificate for {}:{}", address, port);
                    }
                } else {
                    log::error!(
                        "failed to convert certificate to der for {}:{}",
                        address,
                        port
                    );
                }
            } else {
                log::error!("failed to get peer certificate for {}:{}", address, port);
            }
            drop(tls);
        } else {
            log::error!(
                "failed to upgrade tcp stream to tls for {}:{}: {:?}",
                address,
                port,
                upgraded.err()
            );
        }
    } else {
        drop(stream); // close original connection
    }

    log::debug!("grabbing http banner for {} ...", &url);

    let cli = reqwest::Client::builder()
        .no_proxy() // used to set auto_sys_proxy to false, see https://github.com/evilsocket/legba/issues/8
        .danger_accept_invalid_certs(true)
        .build();
    let cli = if let Ok(c) = cli {
        c
    } else {
        log::error!(
            "can't create http client for {}:{}: {:?}",
            address,
            port,
            cli.err()
        );
        return banner;
    };

    let resp = cli
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/45.0.2454.85 Safari/537.36")
        .timeout(timeout)
        .send()
        .await;

    if let Ok(resp) = resp {
        parse_http_response(opts, resp, &mut banner, address, port).await;
    } else {
        log::debug!(
            "can't connect via http client to {}:{}: {:?}",
            address,
            port,
            resp.err()
        );
    }

    banner
}
