# SMTP NTLM mock test server

A minimal Python ([asyncio][1]) SMTP server that speaks `AUTH NTLM` per
[MS-SMTPNTLM][2] and validates the client's NTLMv2 `Type 3` response against
known credentials. Used to exercise legba's `--smtp-mechanism NTLM` path
without spinning up a real Exchange instance.

## Usage

Standalone:

```sh
python3 mock_smtp_ntlm.py
# Listens on :2526 by default with jeff / letmein / LEGBA
```

With Docker Compose:

```sh
docker compose up
# Same defaults, port mapped to host :2526
```

Configuration is via environment variables:

| Variable | Default | Description |
| --- | --- | --- |
| `LEGBA_NTLM_PORT` | `2526` | Bind port (the compose file maps host `2526` to container `25`) |
| `LEGBA_NTLM_USER` | `jeff` | Username the server will accept |
| `LEGBA_NTLM_PASSWORD` | `letmein` | Password the server will accept |
| `LEGBA_NTLM_DOMAIN` | `LEGBA` | NetBIOS domain advertised in `Type 2` |
| `LEGBA_NTLM_STARTTLS` | _(unset)_ | `1` / `on` / `yes` to advertise + accept STARTTLS; `require` to also refuse AUTH on the unencrypted channel |
| `LEGBA_NTLM_TLS_CERT` | _(ephemeral)_ | Path to a PEM cert. When unset, an ephemeral self-signed cert is generated (requires the `cryptography` package). |
| `LEGBA_NTLM_TLS_KEY` | _(ephemeral)_ | Path to the matching PEM key. |

## Smoke tests

Plain (no STARTTLS):

```sh
legba smtp -T 127.0.0.1:2526 -U jeff -P letmein \
    --smtp-mechanism NTLM --smtp-ntlm-domain LEGBA \
    --timeout 5000 --single-match     # positive

legba smtp -T 127.0.0.1:2526 -U jeff -P wrongpass \
    --smtp-mechanism NTLM --smtp-ntlm-domain LEGBA \
    --timeout 5000 --single-match     # negative
```

With STARTTLS (start the mock with `LEGBA_NTLM_STARTTLS=require`):

```sh
legba smtp -T 127.0.0.1:2526 -U jeff -P letmein \
    --smtp-mechanism NTLM --smtp-ntlm-domain LEGBA --smtp-starttls \
    --timeout 10000 --single-match    # positive over TLS

legba smtp -T 127.0.0.1:2526 -U jeff -P letmein \
    --smtp-mechanism PLAIN --smtp-starttls \
    --timeout 10000 --single-match    # positive PLAIN over TLS
```

## Dependencies

- `aiosmtpd` (pip install) — provides the SMTP server stack, including
  STARTTLS upgrade handling on the asyncio Streams API.
- `cryptography` (pip install) — used only to mint an ephemeral self-signed
  cert when no cert/key path is supplied. Skip the install by setting
  `LEGBA_NTLM_TLS_CERT` and `LEGBA_NTLM_TLS_KEY` to a pre-existing pair.

## Implementation notes

- Bundles a self-contained MD4 implementation because hashlib's MD4 is gated
  behind the OpenSSL legacy provider on most modern distributions.
- Uses a fixed server challenge (`0123456789abcdef`) so behaviour is
  deterministic across runs.
- Only NTLMv2 is verified. NTLMv1 connections will get a 535 since the
  ntproof-str validator uses the v2 HMAC chain.

[1]: https://docs.python.org/3/library/asyncio.html
[2]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-smtpntlm/
