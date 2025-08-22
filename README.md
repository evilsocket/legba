<div align="center">

# `legba`

[![Documentation](https://img.shields.io/badge/docs-blue)](https://github.com/evilsocket/legba/blob/main/docs/index.md)
[![Release](https://img.shields.io/github/release/evilsocket/legba.svg?style=flat-square)](https://github.com/evilsocket/legba/releases/latest)
[![Rust Report](https://rust-reportcard.xuri.me/badge/github.com/evilsocket/legba)](https://rust-reportcard.xuri.me/report/github.com/evilsocket/legba)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/legba/ci.yml)](https://github.com/evilsocket/legba/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/legba/blob/master/LICENSE.md)

  <small>Join the project community on our server!</small>
  <br/><br/>
  <a href="https://discord.gg/btZpkp45gQ" target="_blank" title="Join our community!">
    <img src="https://dcbadge.limes.pink/api/server/https://discord.gg/btZpkp45gQ"/>
  </a>

</div>

Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator built with Rust and the Tokio asynchronous runtime in order to achieve
[better performances and stability](https://github.com/evilsocket/legba/blob/main/docs/benchmark.md) while consuming less resources than similar tools.

## Key Features

- **100% Rust** - Legba is entirely written in Rust, does not have native dependencies and can be easily compiled for all operating systems and architectures. 🦀
- **Multi Protocol** - Support for HTTP, DNS, SSH, FTP, SMTP, RDP, VNC, SQL databases, NoSQL, LDAP, Kerberos, SAMBA, SNMP, STOMP, MQTT [and more](https://github.com/evilsocket/legba/blob/main/docs/index.md).
- **High Performance** - Async/concurrent architecture with customizable workers for [maximum speed](https://github.com/evilsocket/legba/blob/main/docs/benchmark.md).
- **Flexible Credentials** - [Multiple input formats](https://github.com/evilsocket/legba/blob/main/docs/usage.md) including wordlist files, ranges, permutations, and expression generators.
- **Smart Session Management** - Save and restore session state to resume interrupted scans.
- **Advanced Rate Control** - Rate limiting, delays, jittering, and retry mechanisms for stealth and stability.
- **Extensible Architecture** - [REST API](https://github.com/evilsocket/legba/blob/main/docs/rest.md), [Model Context Protocol (MCP)](https://github.com/evilsocket/legba/blob/main/docs/mcp.md) server, and custom binary plugin support.
- **Recipe System** - [YAML-based configuration](https://github.com/evilsocket/legba/blob/main/docs/recipes.md) for complex authentication scenarios.
- **Multiple Output Formats** - Export results in various formats for easy integration with other tools.

## Quick Start

Download one of the precompiled binaries from the [project latest release page](https://github.com/evilsocket/legba/releases/latest), or if you're a **Homebrew** user, you can install it with a custom tap:

```bash
brew tap evilsocket/legba https://github.com/evilsocket/legba
brew install evilsocket/legba/legba
```

You are now ready to go! 🚀

```bash
legba smb --target domain.local --username administrator --password wordlist.txt
```

For the usage and the complete list of options [check the project documentation](https://github.com/evilsocket/legba/blob/main/docs/index.md).

## Contributors

<a href="https://github.com/evilsocket/legba/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=evilsocket/legba" alt="Legba project contributors" />
</a>

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/legba&type=Timeline)](https://www.star-history.com/#evilsocket/legba&Timeline)

## License

Legba is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.
