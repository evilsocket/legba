---
title: POP3 brute-force plugin for legba
description: Async POP3 password authentication wordlist attacks with optional SSL or TLS support.
---

POP3 password authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--pop3-ssl` | Enable SSL for POP3 |

## Examples

Insecure:

```sh
legba pop3 \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:110
```

Via SSL:

```sh
legba pop3 \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:995 \
    --pop3-ssl
```