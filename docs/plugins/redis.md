---
title: Redis legacy and ACL brute-force
description: Brute-force Redis legacy password and ACL authentication. Optional SSL or TLS. Single-binary alternative to hydra redis.
---

Redis password authentication, both legacy and ACL based.

## Options

| Name | Description |
| ---- | ----------- | 
| `--redis-ssl` | Enable SSL for Redis. |

## Examples

```sh
legba redis \
    --target localhost:6379 \
    --username admin \
    --password data/passwords.txt
```