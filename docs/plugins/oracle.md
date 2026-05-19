---
title: Oracle DB brute-force plugin for legba
description: Brute-force Oracle Database authentication with configurable target database. Single-binary alternative to hydra oracle.
---

Oracle Password Authentication.

**NOTE**: this is an optional feature that is not compiled by default, enable during compilation with by using `cargo build --release -F oracle`.

## Examples 

```sh
legba oracle \
    --target localhost:1521 \
    --oracle-database SYSTEM \
    --username admin \
    --password data/passwords.txt
```
