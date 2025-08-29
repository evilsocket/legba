use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use dashmap::DashSet;
use trust_dns_resolver::{AsyncResolver, TokioAsyncResolver, config::*};
use x509_parser::prelude::{FromDer, GeneralName, X509Certificate};

use crate::Options;
use crate::Plugin;
use crate::session::{Error, Loot};
use crate::utils::net::{async_tcp_stream, upgrade_tcp_stream_to_tls};

use crate::creds::Credentials;

use super::plugin::PayloadStrategy;

pub(crate) mod options;

super::manager::register_plugin! {
    "dns" => DNS::new()
}

#[derive(Clone)]
pub(crate) struct DNS {
    ips: Vec<IpAddr>,
    opts: options::Options,
    core_options: Options,
    hits: Arc<DashMap<IpAddr, usize>>,
    domains: Arc<DashSet<String>>,
}

impl DNS {
    pub fn new() -> Self {
        DNS {
            ips: vec![],
            opts: options::Options::default(),
            core_options: Options::default(),
            hits: Arc::new(DashMap::new()),
            domains: Arc::new(DashSet::new()),
        }
    }

    async fn get_resolver(&self, timeout: Duration) -> Result<TokioAsyncResolver, Error> {
        if !self.ips.is_empty() {
            let nameserver_group =
                NameServerConfigGroup::from_ips_clear(&self.ips, self.opts.dns_port, true);

            let mut options = ResolverOpts::default();

            options.num_concurrent_reqs = self.core_options.concurrency;
            options.attempts = self.opts.dns_attempts;
            options.timeout = timeout;
            options.shuffle_dns_servers = true;

            Ok(AsyncResolver::tokio(
                ResolverConfig::from_parts(None, vec![], nameserver_group),
                options,
            ))
        } else {
            let (config, mut options) =
                trust_dns_resolver::system_conf::read_system_conf().map_err(|e| e.to_string())?;

            options.attempts = self.opts.dns_attempts;
            options.timeout = timeout;

            Ok(AsyncResolver::tokio(config, options))
        }
    }

    async fn filter(&self, addresses: Vec<IpAddr>) -> Vec<IpAddr> {
        // Some domains are configured to resolve any subdomain, whatever it is, to the same IP. We do
        // this filtering in order too many positives for an address and work around this behaviour.
        let mut filtered = vec![];
        for ip in &addresses {
            let curr_hits = if let Some(mut ip_hits) = self.hits.get_mut(ip) {
                // this ip already has a counter, increment it
                *ip_hits += 1;
                *ip_hits
            } else {
                // first time we see this ip, create the counter for it
                self.hits.insert(ip.to_owned(), 1);
                1
            };

            if curr_hits <= self.opts.dns_max_positives {
                filtered.push(ip.to_owned());
            } else if curr_hits == self.opts.dns_max_positives + 1 {
                // log this just the first time
                log::warn!(
                    "address {} reached {} positives and will be filtered out from further resolutions.",
                    ip,
                    curr_hits
                )
            }
        }

        filtered
    }

    async fn get_additional_tls_loot(
        &self,
        target: &str,
        subdomain: &str,
        resolver: &TokioAsyncResolver,
        timeout: Duration,
    ) -> Vec<Loot> {
        let mut loot = vec![];

        // check if port 443 is open
        let https_address = format!("{}:443", subdomain);
        let stream = match async_tcp_stream(&https_address, "", timeout, false).await {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        // upgrade to TLS
        let tls = match upgrade_tcp_stream_to_tls(stream, subdomain, timeout).await {
            Ok(t) => t,
            Err(_) => return vec![],
        };
        // get certificate
        let cert = match tls.peer_certificate() {
            Ok(Some(cert)) => cert,
            _ => return vec![],
        };
        // convert to DER
        let der = match cert.to_der() {
            Ok(der) => der,
            _ => return vec![],
        };
        // parse again because there's no way to do it directly :/
        let cert = match X509Certificate::from_der(&der) {
            Ok((_, cert)) => cert,
            _ => return vec![],
        };
        // get alternative names / hosts
        let alt_names = match cert.subject_alternative_name() {
            Ok(Some(names)) => names,
            _ => return vec![],
        };

        let check = format!(".{}", target).to_ascii_lowercase();
        for name in alt_names.value.general_names.iter() {
            let tls_domain = match name {
                GeneralName::DNSName(s) => s.to_ascii_lowercase(),
                _ => continue,
            };
            // skip wildcard names and other domains
            if !tls_domain.contains('*') && tls_domain.ends_with(&check) {
                // skip domains that have already been processed
                if !self.domains.contains(&tls_domain) {
                    // try to resolve to ip
                    if let Ok(response) = resolver.lookup_ip(&tls_domain).await {
                        // collect valid IPs
                        let addresses: Vec<IpAddr> =
                            response.iter().filter(|ip| !ip.is_loopback()).collect();

                        if !addresses.is_empty() {
                            log::debug!(
                                "found new domain from tls records: {} -> {:?}",
                                &tls_domain,
                                &addresses
                            );
                            loot.push(Loot::new(
                                "dns",
                                &tls_domain,
                                vec![
                                    (
                                        "addresses".to_owned(),
                                        addresses
                                            .iter()
                                            .map(|a| a.to_string())
                                            .collect::<Vec<String>>()
                                            .join(", "),
                                    ),
                                    ("alt_name_of".to_owned(), subdomain.to_owned()),
                                ],
                            ));
                        }
                    }
                }
            }
        }

        loot
    }
}

#[async_trait]
impl Plugin for DNS {
    fn description(&self) -> &'static str {
        "DNS subdomain enumeration."
    }

    fn payload_strategy(&self) -> PayloadStrategy {
        PayloadStrategy::Single
    }

    async fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.core_options = opts.clone();
        self.opts = opts.dns.clone();
        self.ips = if let Some(resolvers) = opts.dns.dns_resolvers.as_ref() {
            let ips: Vec<IpAddr> = resolvers
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<IpAddr>().unwrap())
                .collect();

            log::info!("using resolvers: {:?}", &ips);

            ips
        } else {
            log::info!("using system resolver");

            vec![]
        };

        Ok(())
    }

    async fn attempt(
        &self,
        creds: &Credentials,
        timeout: Duration,
    ) -> Result<Option<Vec<Loot>>, Error> {
        // make sure we only extract the host from the target
        let target_host = if let Ok(url) = url::Url::parse(&creds.target) {
            url.host_str().unwrap_or(&creds.target).to_string()
        } else {
            // If not a valid URL, treat as hostname directly
            creds.target.clone()
        };

        let subdomain = format!("{}.{}", creds.single(), &target_host).to_lowercase();
        // skip domains that have already been processed
        if self.domains.contains(&subdomain) {
            return Ok(None);
        }

        // each worker will use its own resolver object instance
        let resolver = self.get_resolver(timeout).await?;
        let started_at = std::time::Instant::now();
        // attempt resolving this subdomain to a one or more IP addresses
        if let Ok(response) = resolver.lookup_ip(&subdomain).await {
            let elapsed = started_at.elapsed();
            // collect valid IPs
            let addresses: Vec<IpAddr> = response.iter().filter(|ip| !ip.is_loopback()).collect();
            // Some domains are configured to resolve any subdomain, whatever it is, to the same IP. We do
            // this filtering in order too many positives for an address and work around this behaviour.
            let addresses = self.filter(addresses).await;
            if !addresses.is_empty() {
                let mut loot_data = vec![];
                let addr_data = if self.opts.dns_ip_lookup {
                    // perform reverse lookup of the IPs if we have to
                    let mut parts = vec![];
                    for ip in &addresses {
                        if let Ok(hostname) = dns_lookup::lookup_addr(ip) {
                            if hostname != subdomain {
                                parts.push(format!("{} ({})", ip, hostname));
                            }
                        } else {
                            parts.push(ip.to_string());
                        }
                    }

                    parts.join(", ")
                } else {
                    // just join the IPs
                    addresses
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                };

                loot_data.push(("addresses".to_owned(), addr_data));
                loot_data.push(("time_ms".to_owned(), elapsed.as_millis().to_string()));

                let mut loot = vec![Loot::new("dns", &subdomain, loot_data)];

                // keep track of domains we processed already
                self.domains.insert(subdomain.to_owned());

                if !self.opts.dns_no_https {
                    let more_loot = self
                        .get_additional_tls_loot(&target_host, &subdomain, &resolver, timeout)
                        .await;

                    // keep track of domains we processed already
                    for item in more_loot.iter() {
                        self.domains.insert(item.get_target().to_string());
                    }

                    loot.extend(more_loot);
                }

                return Ok(Some(loot));
            }
        }

        Ok(None)
    }
}
