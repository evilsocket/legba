---
title: RDP brute-force plugin for legba
description: Microsoft Remote Desktop Protocol password brute-force with optional NTLM, admin mode, auto-logon flags, and domain support.
---

Microsoft Remote Desktop.

## Options

| Name | Description |
| ---- | ----------- |
| `--rdp-domain <RDP_DOMAIN>` | Domain name [default: ``] |
| `--rdp-ntlm` | Use a NTLM hash instead of a password |
| `--rdp-admin-mode` | Restricted admin mode |
| `--rdp-auto-logon` | AutoLogon mode in case of SSL negotiation |

## Examples

```sh
legba rdp \
    --target localhost:3389 \
    --username admin \
    --password data/passwords.txt
```