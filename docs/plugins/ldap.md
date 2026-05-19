---
title: LDAP brute-force plugin for legba
description: Async LDAP bind authentication brute-force with domain support. Modern hydra alternative with first-class rate limiting and recipes.
---

LDAP Password Authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--ldap-domain <LDAP_DOMAIN>` | LDAP domain |

## Examples

```sh
legba ldap \
    --target 127.0.0.1:389 \
    --username admin \
    --password @wordlists/passwords.txt \
    --ldap-domain example.org \
    --single-match
```