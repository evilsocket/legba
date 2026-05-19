---
title: legba vs Hydra, Medusa, Ncrack, Patator
description: Side-by-side comparison of legba with THC-Hydra, Medusa, Ncrack, and Patator across speed, protocol coverage, automation, and packaging.
---

# legba vs Hydra, Medusa, Ncrack, Patator

This page compares **legba** with the four most widely used credential bruteforcers in the security community: [THC-Hydra](https://github.com/vanhauser-thc/thc-hydra), [Medusa](https://github.com/jmk-foofus/medusa), [Ncrack](https://nmap.org/ncrack/), and [Patator](https://github.com/lanjelot/patator). Where speed is reported, the numbers are measured. See [Benchmark](benchmark.md) for the full methodology.

## TL;DR

| Question | Answer |
| -------- | ------ |
| Which is **fastest** on common protocols? | **legba**, by 1.5×–55× over THC-Hydra on identical hardware and wordlists (HTTP basic auth, HTTP POST login, SSH, MySQL, MSSQL). [Benchmark methodology.](benchmark.md) |
| Which has the **best AI agent integration**? | **legba** ships a REST API and a Model Context Protocol (MCP) server out of the box. No other tool in this list exposes an MCP server. |
| Which has **no native dependencies**? | **legba** is a single static Rust binary, no libssh/libssl/libpq/etc to link. Hydra, Medusa, and Ncrack are C with multiple C library deps. Patator is Python with per-protocol Python deps. |
| Which has a **YAML recipe / reusable attack config** system? | **legba** (built-in YAML recipes). Patator approximates this via shell aliases; Hydra/Medusa/Ncrack have no equivalent. |
| Which is **actively maintained** (commits in last 12 months)? | legba ✓, Patator ✓ (sporadic), Hydra ✓, Medusa (low activity), Ncrack (low activity). Check upstream repos for current status. |

## Speed comparison (vs THC-Hydra)

All tests on the same machine (M1 Max), same target server on localhost, same 1000-password wordlist with the correct password on the last line. Legba compiled in release mode; Hydra installed via Homebrew. Full reproduction details in [Benchmark](benchmark.md).

| Protocol | Hydra time | Legba time | Speedup |
| -------- | ---------- | ---------- | ------- |
| HTTP basic auth | 7.100 s | **1.560 s** | **4.5×** |
| HTTP POST login (WordPress) | 14.854 s | **5.045 s** | **2.9×** |
| SSH | 7 m 29.85 s | **8.150 s** | **55.1×** |
| MySQL | 9.819 s | **2.542 s** | **3.8×** |
| Microsoft SQL Server | 7.609 s | **4.789 s** | **1.5×** |

Medusa, Ncrack, and Patator were not included in the benchmark run. Anecdotal community reports place Medusa close to Hydra and Patator (Python-based) consistently slower than both; we welcome [reproducible measurements](https://github.com/evilsocket/legba/issues) to add to this table.

## Feature matrix

Legend: ● built-in · ◐ partial / via plugin · ○ not supported

| Capability | legba | Hydra | Medusa | Ncrack | Patator |
| ---------- | :---: | :---: | :----: | :----: | :-----: |
| Async / non-blocking core | ● | ○ | ○ | ◐ | ○ |
| Single static binary (no native deps) | ● | ○ | ○ | ○ | ○ |
| Rate limiting | ● | ◐ | ◐ | ● | ● |
| Per-attempt jitter (anti-detection) | ● | ○ | ○ | ○ | ◐ |
| Resumable sessions | ● | ● | ○ | ● | ○ |
| Wordlist + permutation + range expressions | ● | ◐ | ◐ | ○ | ● |
| Glob expressions for files (e.g. `*.key`) | ● | ○ | ○ | ○ | ◐ |
| YAML recipes (reusable attack configs) | ● | ○ | ○ | ○ | ○ |
| REST API | ● | ○ | ○ | ○ | ○ |
| **Model Context Protocol (MCP) for AI agents** | **●** | ○ | ○ | ○ | ○ |
| Custom binary plugin (wrap any CLI) | ● | ○ | ○ | ○ | ◐ |
| HTTP basic auth | ● | ● | ● | ● | ● |
| HTTP form login (with CSRF token grabbing) | ● | ◐ | ◐ | ○ | ● |
| HTTP NTLMv1 / NTLMv2 | ● | ◐ | ○ | ○ | ○ |
| HTTP page enumeration | ● | ○ | ○ | ○ | ○ |
| HTTP virtual host enumeration | ● | ○ | ○ | ○ | ○ |
| SSH / SFTP | ● | ● | ● | ● | ● |
| RDP | ● | ● | ○ | ● | ○ |
| VNC | ● | ● | ● | ● | ○ |
| SMB / Samba (auth + share enum) | ● | ◐ | ● | ○ | ● |
| LDAP | ● | ● | ● | ○ | ● |
| Kerberos | ● | ○ | ○ | ○ | ◐ |
| MySQL / PostgreSQL / MSSQL / Oracle | ● | ● | ◐ | ○ | ● |
| MongoDB / ScyllaDB / Cassandra | ● | ○ | ○ | ○ | ◐ |
| Redis | ● | ○ | ○ | ○ | ◐ |
| AMQP / MQTT / STOMP | ● | ○ | ○ | ○ | ◐ |
| SNMP v1 / v2 / v3 | ● | ◐ | ○ | ○ | ● |
| DNS subdomain enumeration | ● | ○ | ○ | ○ | ● |
| TCP / UDP port scanner with banners | ● | ○ | ○ | ○ | ○ |
| IRC / Telnet / SOCKS5 | ● | ◐ | ◐ | ◐ | ◐ |
| SMTP / IMAP / POP3 | ● | ● | ● | ● | ● |

Inevitable disclaimer: Hydra, Medusa, Ncrack, and Patator are all mature, well-respected projects and each has features and corner cases that legba does not match yet. The table above reflects documented capabilities of each tool at the time of writing; correct any inaccuracy by [opening an issue](https://github.com/evilsocket/legba/issues).

## When to pick which tool

- **Pick legba** when you want raw throughput, modern Rust ergonomics, a single static binary, AI-agent driveable workflows (REST + MCP), or reusable YAML attack recipes.
- **Pick Hydra** when you need a protocol legba doesn't ship yet (rare) or when you're constrained to tools already installed on a Kali pin.
- **Pick Medusa** when you specifically want its host/user parallelization model.
- **Pick Ncrack** when you're integrating with Nmap and want shared scripting infrastructure.
- **Pick Patator** when you want Python and the ability to write quick custom protocol modules inline.

## Reproducing the benchmark

The benchmark commands and Docker test servers used to produce the numbers above are part of the repository. See [Benchmark](benchmark.md) and the `test-servers/` directory in the source tree.

```bash
git clone https://github.com/evilsocket/legba
cd legba
cargo build --release
# Spin up a test server (example: HTTP basic auth)
docker compose -f test-servers/http-basic.docker-compose.yml up -d
# Run legba
./target/release/legba http.basic -T http://localhost:8080 -U admin -P wordlists/passwords.txt
```

## See also

- [Benchmark](benchmark.md): full methodology, commands, hardware spec.
- [Usage](usage.md): CLI reference and expression syntax.
- [REST API](rest.md) and [MCP](mcp.md): agent and automation surface.
- [FAQ](faq.md): common questions about legba and how it compares.
