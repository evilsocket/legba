---
title: Microsoft SQL Server brute-force plugin
description: Async brute-force of Microsoft SQL Server authentication. 1.5x faster than hydra mssql on identical hardware (see benchmark).
---

Microsoft SQL Server Password Authentication.

## Examples

```sh
legba mssql \
    --username SA \
    --password wordlists/passwords.txt \
    --target localhost:1433
```