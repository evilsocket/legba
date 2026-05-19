---
title: Install legba on Linux, macOS, Windows
description: How to install legba on Linux, macOS, and Windows. Precompiled binaries, Homebrew, building from source, and the official Docker image.
---

# Installation

## Binaries

Download one of the precompiled binaries from the [project latest release page](https://github.com/evilsocket/legba/releases/latest).

## Cargo

Legba is published as a binary crate on [crates.io](https://crates.io/crates/legba), if you have [Cargo installed](https://rustup.rs/) you can:

```sh
cargo install legba
```

This will compile its sources and install the binary in `$HOME/.cargo/bin/legba`.

## Homebrew

If you're a **Homebrew** user, you can install Legba with a custom tap:

```bash
brew tap evilsocket/legba https://github.com/evilsocket/legba
brew install evilsocket/legba/legba
```

## Docker

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

## AI Agent Skill

If you use an AI coding agent (Claude Code, Cursor, Copilot, OpenCode, and [many others](https://skills.sh)), you can install the legba skill to give it full knowledge of the tool — target syntax, credential expressions, plugins, recipes, REST API, and MCP server:

```bash
npx skills add https://github.com/evilsocket/legba --skill legba
```

Once installed, your agent will be able to construct legba commands, write recipes, and configure the REST API or MCP server without needing to look things up manually.

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