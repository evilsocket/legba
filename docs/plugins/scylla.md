---
title: ScyllaDB / Cassandra brute-force
description: Async brute-force of ScyllaDB and Cassandra authentication on the default CQL port 9042.
---

ScyllaDB / Apache Casandra password based authentication.

## Examples

```sh
legba scylla \
    --username cassandra \
    --password wordlists/passwords.txt \
    --target localhost:9042
```
