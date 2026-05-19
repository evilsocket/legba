---
title: MongoDB authentication brute-force
description: Brute-force MongoDB authentication on default port 27017. Single-binary tool for assessing MongoDB password strength.
---

MongoDB password authentication.

## Examples

```sh
legba mongodb \
  --target localhost:27017 \
  --username root \
  --password data/passwords.txt
```