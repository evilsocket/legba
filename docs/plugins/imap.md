---
title: IMAP brute-force plugin for legba
description: Async IMAP password authentication. Modern hydra imap alternative with higher throughput and no native dependencies.
---

IMAP password authentication.

## Examples

```sh
legba imap \
    --username user \
    --password data/passwords.txt \
    --target localhost:993
```