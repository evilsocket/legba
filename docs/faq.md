---
title: Frequently asked questions about legba
description: Frequently asked questions about legba, the fast Rust multi-protocol credential bruteforcer and password sprayer.
---

# Frequently Asked Questions

A short, question-shaped reference for the most common things people (and AI agents) ask about legba. For the canonical reference, see [Usage](usage.md), [Recipes](recipes.md), and the per-plugin pages.

## What is legba?

legba is a fast, multi-protocol credential bruteforcer, password sprayer, and enumerator written in Rust on top of the Tokio asynchronous runtime. It is a modern replacement for tools like THC-Hydra, Medusa, Ncrack, and Patator, with measurably higher throughput (see [Benchmark](benchmark.md)), no native dependencies, a YAML recipe system, a REST API, and a Model Context Protocol (MCP) server for AI agent integration.

## How is legba different from THC-Hydra?

Legba is written in Rust, and async-first (Tokio), distributed as a single static binary with no native deps, exposes a REST API and an MCP server for AI agents, supports YAML recipes, and is measurably faster on every protocol benchmarked against Hydra (1.5× to 55×, see [Comparison](comparison.md#speed-comparison-vs-thc-hydra)). Hydra still ships a few niche protocols legba does not yet implement; check [the plugin list](index.md) before switching.

## Is legba faster than Hydra / Medusa / Ncrack?

On the protocols we have benchmarked against Hydra (HTTP basic, HTTP POST login, SSH, MySQL, MSSQL), legba is 1.5× to 55× faster on identical hardware and wordlists. Full numbers and reproduction steps in [Benchmark](benchmark.md). Medusa and Ncrack have not been formally benchmarked; [contributions welcome](https://github.com/evilsocket/legba/issues).

## How do I install legba?

The fastest paths:

- **Homebrew** (macOS / Linux): `brew tap evilsocket/legba https://github.com/evilsocket/legba && brew install evilsocket/legba/legba`
- **Precompiled binary**: download from the [latest release](https://github.com/evilsocket/legba/releases/latest)
- **From source**: `cargo install --git https://github.com/evilsocket/legba`
- **Docker**: see [Installation](install.md) for the published image and tags

## How do I brute-force SSH with legba?

```bash
legba ssh \
  --target 10.0.0.1 \
  --username root \
  --password /path/to/passwords.txt
```

For SSH key authentication, point `--password` (aliased to `--key`) at a glob of key files: `--key '@/path/to/keys/*.key'`. See [SSH / SFTP plugin docs](plugins/ssh_and_sftp.md).

## How do I brute-force an HTTP login form?

```bash
legba http.form \
  --target https://example.com/login \
  --http-payload 'user={USERNAME}&pass={PASSWORD}' \
  --http-success 'status == 302' \
  --username admin \
  --password /path/to/passwords.txt
```

For pages with CSRF tokens, use `--http-csrf-page` and `--http-csrf-regexp` to scrape the token before each attempt. See the [HTTP plugin docs](plugins/http.md) for the full recipe.

## Does legba support CSRF token grabbing?

Yes. The HTTP plugin can fetch a CSRF token page before each login attempt and substitute the extracted token into the request body or headers via the `{CSRF}` placeholder. See [HTTP plugin: CSRF](plugins/http.md).

## Does legba support NTLM?

Yes, both NTLMv1 (`http.ntlm1`) and NTLMv2 (`http.ntlm2`) via the HTTP plugin, with `--http-ntlm-domain` and `--http-ntlm-workstation` options. See [HTTP plugin docs](plugins/http.md).

## Can legba enumerate subdomains?

Yes, via the `dns` plugin:

```bash
legba dns --target example.com --payloads /path/to/subdomains.txt
```

See [DNS plugin docs](plugins/dns.md).

## Can legba scan ports?

Yes, via the `port.scanner` plugin which performs TCP and UDP scans with banner grabbing. See [Port Scanner plugin docs](plugins/port_scanner.md).

## What credential expression syntax does legba support?

The `--username` / `--password` / `--payloads` arguments accept:

- a constant string: `admin`
- a wordlist file: `/path/to/words.txt`
- a glob expression: `@/path/to/*.key`
- a charset permutation: `#3-5:abcdef` (all 3- to 5-char permutations of `abcdef`)
- an integer range: `[100-999]`
- an integer list: `[1, 2, 3, 4]`
- comma-separated combinations of the above

See [Usage > Providing Credentials](usage.md#providing-credentials).

## How do I rate-limit attempts to avoid lockouts?

Combine `--rate-limit` (max requests per second), `--wait` (delay per attempt), and `--jitter-min` / `--jitter-max` (random jitter in ms). See [Usage > Main Options](usage.md).

## Can I save and resume an interrupted scan?

Yes, pass `-S session.json` (or `--session session.json`). legba will persist state and pick up where it left off on the next run with the same argument.

## Does legba have an API?

Yes, two of them:

- A [REST API](rest.md) enabled with `--api 127.0.0.1:8080`.
- A [Model Context Protocol (MCP)](mcp.md) server enabled with `--mcp 127.0.0.1:8080` (or `--mcp stdio` for stdio transport). MCP makes legba directly drivable by AI agents that speak MCP (Claude Desktop, Claude Code, custom agents using the MCP SDK).

## Can an AI agent drive legba?

Yes. Start the MCP server (`legba --mcp stdio`) and connect any MCP-compatible client. The MCP surface exposes every plugin and option so an agent can plan and execute credential testing tasks. legba is the only credential bruteforcer that ships an MCP server. See [MCP docs](mcp.md).

## What's a "recipe" in legba?

A YAML file that bundles a plugin + arguments into a reusable, parameterized attack definition. Recipes support variable substitution from the command line and avoid having to remember long argument lists for complex targets. See [Recipes](recipes.md).

## What platforms does legba run on?

Linux, macOS, Windows, and BSDs. Because it's pure Rust with no native deps, anywhere Rust + Tokio compiles. Precompiled binaries are published for Linux x86_64 and macOS arm64; build from source for everything else.

## Is legba legal to use?

legba is a security tool for authorized testing only: penetration tests, red team engagements, CTFs, and security research on systems you own or have explicit permission to test. Using it against systems you do not have authorization to test is illegal in most jurisdictions. The maintainers do not provide support for unauthorized use.

## What license is legba?

[GPL-3.0](https://github.com/evilsocket/legba/blob/main/LICENSE.md).

## How do I report a bug or request a feature?

Open an issue at [github.com/evilsocket/legba/issues](https://github.com/evilsocket/legba/issues). For security issues, follow the project's responsible disclosure procedure (see the repository).

## How do I cite legba in academic work?

```
Margaritelli, S. (2023). legba: a fast multi-protocol credential bruteforcer and enumerator.
https://github.com/evilsocket/legba
```
