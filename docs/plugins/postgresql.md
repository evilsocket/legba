---
title: PostgreSQL brute-force plugin for legba
description: Async PostgreSQL authentication brute-force. Handles special characters in credentials via typed connect options.
---

PostgreSQL Password Authentication.

## Examples

```sh
legba pgsql \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:5432  
```