---
title: MySQL brute-force plugin for legba
description: Async MySQL password brute-force. Handles usernames and passwords with special characters via typed connect options. 3.8x faster than hydra.
---

MySQL Password Authentication.

## Examples

```sh
legba mysql \
    --username root \
    --password wordlists/passwords.txt \
    --target localhost:3306
```
