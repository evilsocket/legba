---
title: SMTP brute-force (PLAIN, LOGIN, XOAUTH2, NTLM)
description: Async SMTP password authentication brute-force with PLAIN, LOGIN, XOAUTH2, and NTLM (MS-SMTPNTLM) mechanisms. Modern hydra smtp alternative.
---

SMTP password authentication.

## Options

| Name | Description |
| ---- | ----------- |
| `--smtp-mechanism <SMTP_MECHANISM>` | SMTP authentication mechanism: `PLAIN` (RFC4616), `LOGIN` (obsolete but needed for some providers like office365), `XOAUTH2`, `NTLM` (NTLMv2 per [MS-SMTPNTLM]), or `NTLMv1` [default: `PLAIN`] |
| `--smtp-starttls` | Upgrade the connection with STARTTLS after EHLO before authenticating. Required by most modern submission and Exchange servers. |
| `--smtp-ntlm-domain <SMTP_NTLM_DOMAIN>` | NTLM domain to use when `--smtp-mechanism` is `NTLM` or `NTLMv1` |
| `--smtp-ntlm-workstation <SMTP_NTLM_WORKSTATION>` | NTLM workstation identifier to use when `--smtp-mechanism` is `NTLM` or `NTLMv1`. Doubles as the EHLO host name. |

## Examples

PLAIN auth (default):

```sh
legba smtp \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:25
```

NTLM auth against an Exchange-style SMTP service (per [MS-SMTPNTLM]) over the submission port that requires STARTTLS:

```sh
legba smtp \
    --target mail.example.com:587 \
    --username jeff \
    --password wordlists/passwords.txt \
    --smtp-mechanism NTLM \
    --smtp-ntlm-domain LEGBA \
    --smtp-ntlm-workstation pentest1 \
    --smtp-starttls
```

PLAIN auth over STARTTLS (the common production setup for office365, gmail relays, etc.):

```sh
legba smtp \
    --target smtp.example.com:587 \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --smtp-mechanism PLAIN \
    --smtp-starttls
```

Falling back to NTLMv1 if the server only speaks v1:

```sh
legba smtp \
    --target mail.example.com:25 \
    --username jeff \
    --password wordlists/passwords.txt \
    --smtp-mechanism NTLMv1 \
    --smtp-ntlm-domain LEGBA
```

A reference mock server that implements the NTLM handshake and validates
NTLMv2 responses lives in [`test-servers/smtp-ntlm/`](https://github.com/evilsocket/legba/tree/main/test-servers/smtp-ntlm)
and is useful for verifying the plugin end-to-end without an Exchange lab.

[MS-SMTPNTLM]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-smtpntlm/
