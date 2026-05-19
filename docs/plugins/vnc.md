---
title: VNC password brute-force plugin
description: Async VNC password brute-force on default port 5900. Single-binary modern alternative to hydra vnc.
---

VNC Password Authentication.

## Examples

```sh
legba vnc \
    --target localhost:5901 \
    --password data/passwords.txt
```