<div align="center">

# `legba`

<i>Multiprotocol credentials bruteforcer / password sprayer and enumerator. 🥷</i>

[![Documentation](https://img.shields.io/badge/documentation-blue)](https://github.com/evilsocket/legba/wiki)
[![Release](https://img.shields.io/github/release/evilsocket/legba.svg?style=flat-square)](https://github.com/evilsocket/legba/releases/latest)
[![Crate](https://img.shields.io/crates/v/legba.svg)](https://crates.io/crates/legba)
[![Homebrew](https://img.shields.io/homebrew/v/legba)](https://formulae.brew.sh/formula/legba)
[![Docker](https://img.shields.io/docker/v/evilsocket/legba?logo=docker)](https://hub.docker.com/r/evilsocket/legba)
[![Rust Report](https://rust-reportcard.xuri.me/badge/github.com/evilsocket/legba)](https://rust-reportcard.xuri.me/report/github.com/evilsocket/legba)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/legba/test-on-linux.yml)](https://github.com/evilsocket/legba/actions/workflows/test-on-linux.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/legba/blob/master/LICENSE.md)

  <small>Join the project community on our server!</small>
  <br/><br/>
  <a href="https://discord.gg/btZpkp45gQ" target="_blank" title="Join our community!">
    <img src="https://dcbadge.limes.pink/api/server/https://discord.gg/btZpkp45gQ"/>
  </a>

</div>

Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator built with Rust and the Tokio asynchronous runtime in order to achieve
better performances and stability while consuming less resources than similar tools (see the benchmark below).

For the building instructions, usage and the complete list of options [check the project Wiki](https://github.com/evilsocket/legba/wiki). For a list of [recipes](https://github.com/evilsocket/legba/wiki/Recipes) you can check the [legba cookbook repository](https://github.com/evilsocket/legba-cookbook).

## Supported Protocols/Features:

AMQP (ActiveMQ, RabbitMQ, Qpid, JORAM and Solace), Cassandra/ScyllaDB, DNS subdomain enumeration, FTP, HTTP (basic authentication, NTLMv1, NTLMv2, multipart form, custom requests with CSRF support, files/folders enumeration, virtual host enumeration), IMAP, IRC, Kerberos pre-authentication and user enumeration, LDAP, MongoDB, MQTT, Microsoft SQL, MySQL, Oracle, PostgreSQL, POP3, RDP, Redis, Samba, SSH / SFTP, SMTP, Socks5, STOMP (ActiveMQ, RabbitMQ, HornetQ and OpenMQ), TCP and UDP port scanning with banner grabbing, Telnet, VNC.

## AI Integration

As of version 0.11.0, Legba supports being used by an AI as an **MCP server**, refer to [the documentation on how to configure it](https://github.com/evilsocket/legba/wiki/MCP).

## Benchmark

Here's a benchmark of `legba` versus `thc-hydra` running some common plugins, both targeting the same test servers on localhost. The benchmark has been executed on a macOS laptop with an M1 Max CPU, using a wordlist of 1000 passwords with the correct one being on the last line. Legba was compiled in release mode, Hydra compiled and installed via [brew formula](https://formulae.brew.sh/formula/hydra).

Far from being an exhaustive benchmark (some legba features are simply not supported by hydra, such as CSRF token grabbing), this table still gives a clear idea of how using an asynchronous runtime can drastically improve performances.

| Test Name | Hydra Tasks | Hydra Time | Legba Tasks | Legba Time |
| --------- | ----------- | ---------- | ----------- | ---------- |
| HTTP basic auth | 16 | 7.100s | 10 | 1.560s **(🚀 4.5x faster)** |
| HTTP POST login (wordpress) | 16 | 14.854s | 10 | 5.045s **(🚀 2.9x faster)** |
| SSH | 16 | 7m29.85s * | 10 | 8.150s **(🚀 55.1x faster)** |
| MySQL | 4 ** | 9.819s | 4 ** | 2.542s **(🚀 3.8x faster)** |
| Microsoft SQL | 16 | 7.609s | 10 | 4.789s **(🚀 1.5x faster)** |

<sup>* While this result would suggest a default delay between connection attempts used by Hydra. I've tried to study the source code to find such delay but to my knowledge there's none. For some reason it's simply very slow.</sup><br/>
<sup>** For MySQL hydra automatically reduces the amount of tasks to 4, therefore legba's concurrency level has been adjusted to 4 as well.</sup>

## Contributors

<a href="https://github.com/evilsocket/legba/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=evilsocket/legba" alt="Legba project contributors" />
</a>

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/legba&type=Timeline)](https://www.star-history.com/#evilsocket/legba&Timeline)


## License

Legba is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.
