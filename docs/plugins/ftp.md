---
title: FTP brute-force plugin for legba
description: Async FTP password authentication wordlist attacks. Single-binary alternative to hydra ftp with measurably higher throughput.
---

FTP password authentication.

## Examples

Password Authentication:

```sh
legba ftp \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:21
```