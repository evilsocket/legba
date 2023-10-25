`Legba` is a multiprotocol credentials bruteforcer / password sprayer and enumerator built with Rust and the Tokio asynchronous runtime in order to achieve
better performances and stability while consuming less resources than similar tools.

**Work in progress:** while the tool is functioning well overall, it still requires some testing and the integration of more protocols. If you want to contribute with code and/or testing, feel free to check the list of TODOs with `grep -ri --include "*.rs" TODO` ^_^

For the building instructions, usage and the list of supported protocols [check the project Wiki](https://github.com/evilsocket/legba/wiki).

## License

Legba was made with ♥  by [Simone Margaritelli](https://www.evilsocket.net/) and it's released under the GPL 3 license.

To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.