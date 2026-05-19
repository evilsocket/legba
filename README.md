<div align="center">

# `legba`

[![Documentation](https://img.shields.io/badge/docs-blue)](https://legba.evilsocket.net/)
[![Release](https://img.shields.io/github/release/evilsocket/legba.svg?style=flat-square)](https://github.com/evilsocket/legba/releases/latest)
[![Rust Report](https://rust-reportcard.xuri.me/badge/github.com/evilsocket/legba)](https://rust-reportcard.xuri.me/report/github.com/evilsocket/legba)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/legba/ci.yml)](https://github.com/evilsocket/legba/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/legba/blob/master/LICENSE.md)
![Human Coded](https://img.shields.io/badge/human-coded-brightgreen?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBjbGFzcz0ibHVjaWRlIGx1Y2lkZS1wZXJzb24tc3RhbmRpbmctaWNvbiBsdWNpZGUtcGVyc29uLXN0YW5kaW5nIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjUiIHI9IjEiLz48cGF0aCBkPSJtOSAyMCAzLTYgMyA2Ii8+PHBhdGggZD0ibTYgOCA2IDIgNi0yIi8+PHBhdGggZD0iTTEyIDEwdjQiLz48L3N2Zz4=)
 
  <small>Join the project community on our server!</small>
  <br/><br/>
  <a href="https://discord.gg/btZpkp45gQ" target="_blank" title="Join our community!">
    <img src="https://dcbadge.limes.pink/api/server/https://discord.gg/btZpkp45gQ"/>
  </a>

</div>

Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator built with Rust and the Tokio asynchronous runtime in order to achieve
[better performances and stability](https://legba.evilsocket.net/benchmark/) while consuming less resources than similar tools.

## Key Features

- **100% Rust** - Legba is entirely written in Rust, does not have native dependencies and can be easily compiled for all operating systems and architectures. 🦀
- **Multi Protocol** - Support for HTTP, DNS, SSH, FTP, SMTP, RDP, VNC, SQL databases, NoSQL, LDAP, Kerberos, SAMBA, SNMP, STOMP, MQTT [and more](https://legba.evilsocket.net/).
- **High Performance** - Async/concurrent architecture with customizable workers for [maximum speed](https://legba.evilsocket.net/benchmark/).
- **Flexible Credentials** - [Multiple input formats](https://legba.evilsocket.net/usage/) including wordlist files, ranges, permutations, and expression generators.
- **Smart Session Management** - Save and restore session state to resume interrupted scans.
- **Advanced Rate Control** - Rate limiting, delays, jittering, and retry mechanisms for stealth and stability.
- **AI Ready** - [REST API](https://legba.evilsocket.net/rest/), [Model Context Protocol (MCP)](https://legba.evilsocket.net/mcp/) server, and custom binary plugin support.
- **Recipe System** - [YAML-based configuration](https://legba.evilsocket.net/recipes/) for complex authentication scenarios.
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

For the usage and the complete list of options [check the project documentation](https://legba.evilsocket.net/).

## Contributors

<a href="https://github.com/evilsocket/legba/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=evilsocket/legba" alt="Legba project contributors" />
</a>

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/legba&type=Timeline)](https://www.star-history.com/#evilsocket/legba&Timeline)

## License

Legba is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.
