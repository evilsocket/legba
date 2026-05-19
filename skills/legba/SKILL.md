---
name: legba
description: Use this skill when the user wants to brute-force credentials, spray passwords, or enumerate services/subdomains against any network protocol (HTTP, SSH, FTP, SMB, RDP, databases, mail protocols, DNS, etc.) using legba. Also use it when the user asks how to use legba, how to write a recipe, how to configure the REST API or MCP server, or asks for help constructing a legba command.
---

# legba

legba is a fast, multi-protocol credential bruteforcer, password sprayer, and enumerator written in Rust on top of the Tokio async runtime. It is a modern replacement for THC-Hydra, Medusa, Ncrack, and Patator — benchmarked at 4.5× faster on HTTP basic auth, 55× faster on SSH, 3.8× on MySQL vs. Hydra on identical hardware. It ships as a single static binary with no native dependencies and supports 30+ protocol plugins.

Full documentation: https://legba.evilsocket.net/
GitHub: https://github.com/evilsocket/legba

## Installation

```bash
# Precompiled binary (Linux/macOS) — recommended
# Download the latest release from https://github.com/evilsocket/legba/releases

# Homebrew (macOS / Linux)
brew tap evilsocket/legba https://github.com/evilsocket/legba
brew install legba

# Cargo
cargo install legba

# Docker
docker run ghcr.io/evilsocket/legba [args]
```

Full install instructions: https://legba.evilsocket.net/install/

## Core Concepts

### Target Syntax

| Format | Example |
|---|---|
| Single host | `192.168.1.1` |
| Hostname | `example.com` |
| IP range | `192.168.1.1-192.168.1.254` |
| CIDR | `192.168.1.0/24` |
| IPv6 | `[::1]` |
| File of targets | `@targets.txt` |
| Comma-separated | `host1,host2,host3` |

### Credential Expression Syntax

Credentials (username, password, payload) are flexible expressions, not just plain wordlist paths.

| Expression | Meaning |
|---|---|
| `admin` | A single constant value |
| `@wordlist.txt` | One entry per line from a file |
| `@/path/to/*.txt` | Glob — load all matching files |
| `{user}` | Template — substitute the current username into the password expression |
| `[0-9999]` | Integer range, zero-padded automatically |
| `word#3` | Permutations: `word` with all 3-character suffixes |
| `a,b,c` | Explicit comma-separated list |

### Iteration Modes

By default legba iterates over passwords for each username. Change with `--iterate` (`-I`):

```bash
-I user      # iterate over usernames for each password (password spray)
-I password  # default: iterate over passwords for each username (brute-force)
```

## Key CLI Options

| Flag | Description |
|---|---|
| `--target` | Target host/expression (required) |
| `--username` | Username or credential expression |
| `--password` | Password or credential expression |
| `--concurrency` | Number of parallel workers (default: 10) |
| `--rate-limit` | Max requests per second (e.g. `--rate-limit 5`) |
| `--timeout` | Connection timeout in seconds |
| `--retry-times` | Number of retries on failure |
| `--jitter-min/max` | Add random delay (ms) between attempts |
| `--session` | Path to session file for save/resume |
| `--output` | Output file path |
| `--output-format` | `text` (default), `csv`, or `jsonl` |
| `--single-match` | Stop after the first successful credential |
| `--iterate` / `-I` | Iteration strategy: `user` or `password` |
| `--api` | Start REST API on `host:port` |
| `--mcp` | Start MCP server (`host:port` for SSE, `stdio` for stdio mode) |

Full usage reference: https://legba.evilsocket.net/usage/

## Supported Plugins

**Before generating a command for a specific plugin, fetch its documentation page** to get the correct flags and examples.

| Plugin(s) | Description | Docs |
|---|---|---|
| `http`, `http.basic`, `http.form`, `http.ntlm1`, `http.ntlm2`, `http.enum`, `http.vhost` | HTTP auth (basic, form with CSRF, NTLMv1/v2), page enumeration, vhost enumeration | https://legba.evilsocket.net/plugins/http/ |
| `ssh`, `sftp` | Password and private-key authentication | https://legba.evilsocket.net/plugins/ssh_and_sftp/ |
| `ftp` | FTP password auth | https://legba.evilsocket.net/plugins/ftp/ |
| `smtp` | SMTP auth (PLAIN, LOGIN, XOAUTH2, NTLM, NTLMv1); STARTTLS | https://legba.evilsocket.net/plugins/smtp/ |
| `imap` | IMAP password auth | https://legba.evilsocket.net/plugins/imap/ |
| `pop3` | POP3 password auth, optional SSL | https://legba.evilsocket.net/plugins/pop3/ |
| `rdp` | RDP password auth, NTLM hash, admin/auto-logon modes | https://legba.evilsocket.net/plugins/rdp/ |
| `vnc` | VNC password auth | https://legba.evilsocket.net/plugins/vnc/ |
| `smb`, `smb.shares` | SMB/Samba credential brute-force and share enumeration | https://legba.evilsocket.net/plugins/samba/ |
| `ldap` | LDAP bind auth | https://legba.evilsocket.net/plugins/ldap/ |
| `kerberos` | Kerberos 5 pre-auth brute-force and user enumeration | https://legba.evilsocket.net/plugins/kerberos/ |
| `mysql` | MySQL auth | https://legba.evilsocket.net/plugins/mysql/ |
| `pgsql` | PostgreSQL auth | https://legba.evilsocket.net/plugins/postgresql/ |
| `mssql` | Microsoft SQL Server auth | https://legba.evilsocket.net/plugins/mssql/ |
| `oracle` | Oracle DB auth (requires `--features oracle` at build time) | https://legba.evilsocket.net/plugins/oracle/ |
| `mongodb` | MongoDB password auth | https://legba.evilsocket.net/plugins/mongodb/ |
| `scylla` | ScyllaDB / Apache Cassandra auth | https://legba.evilsocket.net/plugins/scylla/ |
| `redis` | Redis legacy and ACL auth, optional SSL | https://legba.evilsocket.net/plugins/redis/ |
| `amqp` | AMQP brokers: ActiveMQ, RabbitMQ, Qpid, JORAM, Solace | https://legba.evilsocket.net/plugins/amqp/ |
| `mqtt` | MQTT v3/v5, optional TLS | https://legba.evilsocket.net/plugins/mqtt/ |
| `stomp` | STOMP brokers: ActiveMQ, RabbitMQ, HornetQ, OpenMQ | https://legba.evilsocket.net/plugins/stomp/ |
| `snmp1`, `snmp2`, `snmp3` | SNMP v1/v2 community string enum, v3 username/password enum, OID tree walking | https://legba.evilsocket.net/plugins/snmp/ |
| `irc` | IRC password auth, optional TLS | https://legba.evilsocket.net/plugins/irc/ |
| `telnet` | Telnet auth with configurable login/password/shell prompts | https://legba.evilsocket.net/plugins/telnet/ |
| `dns` | DNS subdomain enumeration, custom resolvers, HTTPS cert fetch | https://legba.evilsocket.net/plugins/dns/ |
| `port.scanner` | TCP/UDP port scanner with banner grabbing and HTTP/S header grabs | https://legba.evilsocket.net/plugins/port_scanner/ |
| `socks5` | SOCKS5 username/password auth | https://legba.evilsocket.net/plugins/socks5/ |
| `cmd` | Wrap any external CLI tool; detect success via exit code or stdout pattern | https://legba.evilsocket.net/plugins/custom_binary/ |

## Recipe System

Recipes are YAML files that define reusable, parameterized attack configurations. They support variable substitution (`{$var or default}`) and resource embedding relative to the recipe path.

```bash
legba --recipe attack.yaml
# Override recipe variables at runtime:
legba --recipe attack.yaml --set target=192.168.1.1 --set wordlist=passwords.txt
```

Recipes are ideal for complex flows: CSRF token grabbing, multi-step auth, custom headers, or repeatable pentest engagements.

Full recipe reference and examples: https://legba.evilsocket.net/recipes/

## REST API

Start an HTTP API alongside an attack session to query status, list running sessions, and stop them programmatically:

```bash
legba http.basic --target example.com --username admin --password @pass.txt --api 127.0.0.1:8080
```

Full REST API reference: https://legba.evilsocket.net/rest/

## MCP Server

legba is the only credential bruteforcer with a built-in Model Context Protocol (MCP) server, allowing AI agents to drive it programmatically:

```bash
# SSE mode (Claude Desktop, Cline, etc.)
legba --mcp 127.0.0.1:9090

# stdio mode (local agent pipelines)
legba --mcp stdio
```

Full MCP setup and agent configuration snippets: https://legba.evilsocket.net/mcp/

## Session Management

Save progress and resume interrupted attacks:

```bash
# Start with session tracking
legba ssh --target 10.0.0.1 --username root --password @pass.txt --session /tmp/my.session

# Resume later (same command, session file is detected automatically)
legba ssh --target 10.0.0.1 --username root --password @pass.txt --session /tmp/my.session
```

## Where to Look for More

- **FAQ and common attack recipes**: https://legba.evilsocket.net/faq/
- **Speed comparison vs. Hydra/Medusa/Ncrack**: https://legba.evilsocket.net/comparison/
- **Reproducible benchmarks**: https://legba.evilsocket.net/benchmark/
- **Full documentation index**: https://legba.evilsocket.net/
