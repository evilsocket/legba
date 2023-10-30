<p align="center">
  <a href="https://github.com/evilsocket/legba/releases/latest"><img alt="Release" src="https://img.shields.io/github/release/evilsocket/legba.svg?style=flat-square"></a>
  <a href="https://crates.io/crates/legba"><img alt="Crate" src="https://img.shields.io/crates/v/legba.svg"></a>
  <a href="https://hub.docker.com/r/evilsocket/legba"><img alt="Docker Hub" src="https://img.shields.io/docker/v/evilsocket/legba?logo=docker"></a>
  <a href="https://rust-reportcard.xuri.me/report/github.com/evilsocket/legba"><img alt="Rust Report" src="https://rust-reportcard.xuri.me/badge/github.com/evilsocket/legba"></a>
  <a href="https://github.com/evilsocket/legba/blob/master/LICENSE.md"><img alt="Software License" src="https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square"></a>
</p>

`Legba` is a multiprotocol credentials bruteforcer / password sprayer and enumerator built with Rust and the Tokio asynchronous runtime in order to achieve
better performances and stability while consuming less resources than similar tools (see the benchmark below).

For the building instructions, usage and the complete list of options [check the project Wiki](https://github.com/evilsocket/legba/wiki).

## Supported Protocols/Features:

AMQP (ActiveMQ, RabbitMQ, Qpid, JORAM and Solace), DNS subdomain enumeration, FTP, HTTP (basic authentication, NTLMv1, NTLMv2, multipart form, custom request and CSRF support), IMAP, Kerberos pre-authentication and user enumeration, LDAP, MongoDB, Microsoft SQL, MySQL, Oracle, PostgreSQL, POP3, RDP, Redis, SSH / SFTP, SMTP, STOMP (ActiveMQ, RabbitMQ, HornetQ and OpenMQ), Telnet, VNC.

## Benchmark

Here's a benchmark of `legba` versus `thc-hydra` running some common plugins, both targeting the same test servers on localhost. The benchmark has been executed on a macOS laptop with an M1 Max CPU, using a wordlist of 1000 passwords with the correct one being on the last line. Legba was compiled in release mode, Hydra compiled and installed via [brew formula](https://formulae.brew.sh/formula/hydra).

Far from being an exhaustive benchmark (some legba features are simply not supported by hydra, such as CSRF token grabbing), this table still gives a clear idea of how using an asynchronous runtime can drastically improve performances.

| Test Name | Hydra Tasks | Hydra CPU | Hydra Time | Legba Tasks | Legba CPU | Legba Time |
| --------- | ----------- | --------- | ---------- | ----------- | --------- | ---------- | 
| HTTP basic auth | 16 | 3% | 7.100s | 10 | 3% | 1.560s **(🚀 4.5x faster)** |
| HTTP POST login (wordpress) | 16 | 4% | 14.854s | 10 | 2% | 5.045s **(🚀 2.9x faster)** |
| SSH | 16 | 0% | 7m29.85s * | 10 | 10% | 8.150s **(🚀 55.1x faster)** |
| MySQL | 4 ** | 162% | 9.819s | 4 ** | 47% | 2.542s **(🚀 3.8x faster)** |
| Microsoft SQL | 16 | 2% | 7.609s | 16 | 2% | 4.789s **(🚀 1.5x faster)** |

<sup>* While this result would suggest a default delay between connection attempts used by Hydra. I've tried to study the source code to find such delay but to my knowledge there's none. For some reason it's simply very slow.</sup><br/>
<sup>** For MySQL hydra automatically reduces the amount of tasks to 4, therefore legba's concurrency level has been adjusted to 4 as well.</sup>

## License

Legba was made with ♥  by [Simone Margaritelli](https://www.evilsocket.net/) and it's released under the GPL 3 license.

To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.