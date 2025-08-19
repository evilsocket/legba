# Installation

### Cargo

Legba is published as a binary crate on [crates.io](https://crates.io/crates/legba), if you have [Cargo installed](https://rustup.rs/) you can:

```sh
cargo install legba
```

This will compile its sources and install the binary in `$HOME/.cargo/bin/legba`.

### Homebrew

On macOS:

```sh
brew install legba
```

### Docker

For any OS supporting docker, an image is available on [Docker Hub](https://hub.docker.com/r/evilsocket/legba):

```sh
docker run -it evilsocket/legba -h 
```

When using wordlist files, remember to share them via a docker volume. Moreover you'll want to use the host network in order to reach the target, for instance:

```sh
docker run \
  -v $(pwd):/data \ # shared the current directory as /data inside the container
  --network host \ # docker will use the same network of the host
  -it evilsocket/legba:latest \
  ssh --username root --password /data/your-wordlist.txt --target 192.168.1.1
```

## Building

### Sources

Building the project from sources requires [Rust to be installed](https://rustup.rs/). After cloning this repository you can build it with:

```sh
cargo build --release
```

The binary will be compiled inside the `./target/release` folder.

### Docker Image

Alternatively it is possible to build a Docker container:

```sh
docker build -t legba .
```