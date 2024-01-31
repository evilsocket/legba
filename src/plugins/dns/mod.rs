use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use trust_dns_resolver::{config::*, AsyncResolver, TokioAsyncResolver};

use crate::session::{Error, Loot};
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

pub(crate) mod options;

#[ctor]
fn register() {
    crate::plugins::manager::register("dns", Box::new(DNS::new()));
}

#[derive(Clone)]
pub(crate) struct DNS {
    resolver: Option<TokioAsyncResolver>,
    opts: options::Options,
}

impl DNS {
    pub fn new() -> Self {
        DNS {
            resolver: None,
            opts: options::Options::default(),
        }
    }
}

#[async_trait]
impl Plugin for DNS {
    fn description(&self) -> &'static str {
        "DNS subdomain enumeration."
    }

    fn single_credential(&self) -> bool {
        true
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        self.opts = opts.dns.clone();
        self.resolver = Some(if let Some(resolvers) = opts.dns.dns_resolvers.as_ref() {
            let ips: Vec<IpAddr> = resolvers
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<IpAddr>().unwrap())
                .collect();

            log::info!("using resolvers: {:?}", &ips);

            let nameserver_group =
                NameServerConfigGroup::from_ips_clear(&ips, opts.dns.dns_port, true);

            let mut options = ResolverOpts::default();

            options.num_concurrent_reqs = opts.concurrency;
            options.attempts = opts.dns.dns_attempts;
            options.timeout = Duration::from_millis(opts.timeout);
            options.shuffle_dns_servers = true;

            AsyncResolver::tokio(
                ResolverConfig::from_parts(None, vec![], nameserver_group),
                options,
            )
        } else {
            log::info!("using system resolver");

            AsyncResolver::tokio_from_system_conf().map_err(|e| e.to_string())?
        });

        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, _: Duration) -> Result<Option<Loot>, Error> {
        let subdomain = format!("{}.{}", creds.single(), &creds.target);
        // attempt resolving this subdomain to a one or more IP addresses
        if let Ok(response) = self.resolver.as_ref().unwrap().lookup_ip(&subdomain).await {
            // collect valid IPs
            let addresses: Vec<IpAddr> = response.iter().filter(|ip| !ip.is_loopback()).collect();
            if !addresses.is_empty() {
                let loot_data = if self.opts.dns_ip_lookup {
                    // perform reverse lookup of the IPs if we have to
                    let mut parts = vec![];

                    for ip in &addresses {
                        if let Ok(hostname) = dns_lookup::lookup_addr(ip) {
                            if hostname != subdomain {
                                parts.push(format!("{} ({})", ip.to_string(), hostname));
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

                return Ok(Some(Loot::new(
                    "dns",
                    &subdomain,
                    [("addresses".to_owned(), loot_data)],
                )));
            }
        }

        Ok(None)
    }
}
