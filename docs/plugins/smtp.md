---
title: SMTP brute-force (PLAIN, LOGIN, XOAUTH2)
description: Async SMTP password authentication brute-force with PLAIN, LOGIN, and XOAUTH2 mechanisms. Modern hydra smtp alternative.
---

SMTP password authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--smtp-mechanism <SMTP_MECHANISM>` | SMTP authentication mechanism, can be `PLAIN` (RFC4616), `LOGIN` (obsolete but needed for some providers like office365) or `XOAUTH2` [default: `PLAIN`] |

## Examples

```sh
legba smtp \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:25
```