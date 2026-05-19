#!/usr/bin/env python3
"""Minimal SMTP server that speaks AUTH NTLM ([MS-SMTPNTLM]) and AUTH PLAIN,
with optional STARTTLS, used to exercise legba's SMTP plugin against a
deterministic, isolated target.

Built on `aiosmtpd` (the maintained successor to the stdlib `smtpd` module)
because rolling STARTTLS by hand on the asyncio Streams API is fragile.

Configuration via environment variables:
    LEGBA_NTLM_PORT       bind port (default 2526)
    LEGBA_NTLM_USER       username to accept (default jeff)
    LEGBA_NTLM_PASSWORD   password to accept (default letmein)
    LEGBA_NTLM_DOMAIN     NetBIOS domain we advertise + expect (default LEGBA)
    LEGBA_NTLM_STARTTLS   "1" to advertise + handle STARTTLS, "require" to
                          also refuse AUTH on the unencrypted channel
                          (default off)
    LEGBA_NTLM_TLS_CERT   path to a PEM cert (default: ephemeral self-signed)
    LEGBA_NTLM_TLS_KEY    path to a PEM key
"""

from __future__ import annotations

import asyncio
import base64
import datetime
import hmac
import logging
import os
import ssl
import struct
import sys
import tempfile
from typing import Optional

from aiosmtpd.controller import Controller
from aiosmtpd.smtp import SMTP as AioSMTP, AuthResult, auth_mechanism


# ---------------------------------------------------------------------------
# Pure-Python MD4 (RFC 1320). Bundled because hashlib's MD4 is gated behind
# the OpenSSL legacy provider on most modern distributions.
# ---------------------------------------------------------------------------

_MASK = 0xFFFFFFFF


def _rotl(x: int, n: int) -> int:
    x &= _MASK
    return ((x << n) & _MASK) | (x >> (32 - n))


def md4(message: bytes) -> bytes:
    msg = bytearray(message)
    orig_bits = len(msg) * 8
    msg.append(0x80)
    while len(msg) % 64 != 56:
        msg.append(0)
    msg += orig_bits.to_bytes(8, "little")

    a, b, c, d = 0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476

    def f(x, y, z): return (x & y) | ((~x) & z) & _MASK
    def g(x, y, z): return (x & y) | (x & z) | (y & z)
    def h(x, y, z): return x ^ y ^ z

    for i in range(0, len(msg), 64):
        x = struct.unpack("<16I", bytes(msg[i : i + 64]))
        aa, bb, cc, dd = a, b, c, d

        for k, s in (
            (0, 3), (1, 7), (2, 11), (3, 19),
            (4, 3), (5, 7), (6, 11), (7, 19),
            (8, 3), (9, 7), (10, 11), (11, 19),
            (12, 3), (13, 7), (14, 11), (15, 19),
        ):
            t = (a + f(b, c, d) + x[k]) & _MASK
            a = _rotl(t, s)
            a, b, c, d = d, a, b, c

        for k, s in (
            (0, 3), (4, 5), (8, 9), (12, 13),
            (1, 3), (5, 5), (9, 9), (13, 13),
            (2, 3), (6, 5), (10, 9), (14, 13),
            (3, 3), (7, 5), (11, 9), (15, 13),
        ):
            t = (a + g(b, c, d) + x[k] + 0x5A827999) & _MASK
            a = _rotl(t, s)
            a, b, c, d = d, a, b, c

        for k, s in (
            (0, 3), (8, 9), (4, 11), (12, 15),
            (2, 3), (10, 9), (6, 11), (14, 15),
            (1, 3), (9, 9), (5, 11), (13, 15),
            (3, 3), (11, 9), (7, 11), (15, 15),
        ):
            t = (a + h(b, c, d) + x[k] + 0x6ED9EBA1) & _MASK
            a = _rotl(t, s)
            a, b, c, d = d, a, b, c

        a = (a + aa) & _MASK
        b = (b + bb) & _MASK
        c = (c + cc) & _MASK
        d = (d + dd) & _MASK

    return struct.pack("<4I", a, b, c, d)


# ---------------------------------------------------------------------------
# NTLM message construction and validation.
# ---------------------------------------------------------------------------

NTLM_SIG = b"NTLMSSP\x00"
SERVER_CHALLENGE = bytes.fromhex("0123456789abcdef")


def make_type2(domain: str) -> bytes:
    target_name = domain.encode("utf-16-le")
    av_nb_domain = struct.pack("<HH", 2, len(target_name)) + target_name
    av_eol = struct.pack("<HH", 0, 0)
    av_pairs = av_nb_domain + av_eol

    flags = (
        0x00000001  # NEGOTIATE_UNICODE
        | 0x00000200  # NEGOTIATE_NTLM
        | 0x00010000  # TARGET_TYPE_DOMAIN
        | 0x00800000  # NEGOTIATE_TARGET_INFO
    )

    header_len = 48
    target_offset = header_len
    av_offset = target_offset + len(target_name)

    buf = bytearray()
    buf += NTLM_SIG
    buf += struct.pack("<I", 2)
    buf += struct.pack("<HHI", len(target_name), len(target_name), target_offset)
    buf += struct.pack("<I", flags)
    buf += SERVER_CHALLENGE
    buf += b"\x00" * 8
    buf += struct.pack("<HHI", len(av_pairs), len(av_pairs), av_offset)
    buf += target_name
    buf += av_pairs
    return bytes(buf)


def parse_type3(blob: bytes) -> dict:
    if blob[:8] != NTLM_SIG:
        raise ValueError("not an NTLM message")
    mtype = struct.unpack_from("<I", blob, 8)[0]
    if mtype != 3:
        raise ValueError(f"expected type 3, got {mtype}")

    def field(off: int) -> bytes:
        ln, _, ofs = struct.unpack_from("<HHI", blob, off)
        return blob[ofs : ofs + ln]

    return {
        "lm_response": field(12),
        "nt_response": field(20),
        "domain": field(28).decode("utf-16-le", errors="replace"),
        "user": field(36).decode("utf-16-le", errors="replace"),
        "workstation": field(44).decode("utf-16-le", errors="replace"),
    }


def ntlmv2_verify(parsed: dict, server_challenge: bytes, expected_password: str) -> bool:
    nt = parsed["nt_response"]
    if len(nt) < 16:
        return False
    nt_proof_str = nt[:16]
    blob = nt[16:]

    nt_hash = md4(expected_password.encode("utf-16-le"))
    user_domain = (parsed["user"].upper() + parsed["domain"]).encode("utf-16-le")
    ntlmv2_hash = hmac.new(nt_hash, user_domain, "md5").digest()
    expected = hmac.new(ntlmv2_hash, server_challenge + blob, "md5").digest()
    return hmac.compare_digest(expected, nt_proof_str)


# ---------------------------------------------------------------------------
# TLS setup.
# ---------------------------------------------------------------------------


def _ephemeral_self_signed() -> ssl.SSLContext:
    """In-memory self-signed cert for STARTTLS testing."""
    try:
        from cryptography import x509
        from cryptography.hazmat.primitives import hashes, serialization
        from cryptography.hazmat.primitives.asymmetric import rsa
        from cryptography.x509.oid import NameOID
    except ImportError as exc:
        raise RuntimeError(
            "STARTTLS requires the 'cryptography' package or pre-generated "
            "LEGBA_NTLM_TLS_CERT/KEY pair."
        ) from exc

    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    subject = issuer = x509.Name(
        [x509.NameAttribute(NameOID.COMMON_NAME, "mock-smtp-ntlm")]
    )
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(subject)
        .issuer_name(issuer)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now)
        .not_valid_after(now + datetime.timedelta(days=1))
        .sign(key, hashes.SHA256())
    )

    cert_pem = cert.public_bytes(serialization.Encoding.PEM)
    key_pem = key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.TraditionalOpenSSL,
        encryption_algorithm=serialization.NoEncryption(),
    )
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    with tempfile.NamedTemporaryFile(delete=False, suffix=".pem") as cf:
        cf.write(cert_pem)
        cert_path = cf.name
    with tempfile.NamedTemporaryFile(delete=False, suffix=".pem") as kf:
        kf.write(key_pem)
        key_path = kf.name
    ctx.load_cert_chain(certfile=cert_path, keyfile=key_path)
    return ctx


def _load_tls_context() -> Optional[ssl.SSLContext]:
    mode = os.environ.get("LEGBA_NTLM_STARTTLS", "").lower()
    if mode in ("", "0", "off", "false", "no"):
        return None
    cert = os.environ.get("LEGBA_NTLM_TLS_CERT")
    key = os.environ.get("LEGBA_NTLM_TLS_KEY")
    if cert and key:
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain(certfile=cert, keyfile=key)
        return ctx
    return _ephemeral_self_signed()


# ---------------------------------------------------------------------------
# SMTP server.
# ---------------------------------------------------------------------------


CFG = {
    "user": os.environ.get("LEGBA_NTLM_USER", "jeff"),
    "password": os.environ.get("LEGBA_NTLM_PASSWORD", "letmein"),
    "domain": os.environ.get("LEGBA_NTLM_DOMAIN", "LEGBA"),
}


class NtlmSMTP(AioSMTP):
    """aiosmtpd SMTP with a custom AUTH NTLM mechanism."""

    @auth_mechanism("NTLM")
    async def auth_NTLM(self, _arg, args):
        log = logging.getLogger("smtp.ntlm")

        # Ask the client for the Type 1 message. aiosmtpd's challenge_auth
        # already returns the base64-decoded client response.
        type1 = await self.challenge_auth("", encode_to_b64=False)
        if not type1:
            return AuthResult(success=False, handled=False, message="535 5.7.8 invalid NTLM Type 1")

        type2 = make_type2(CFG["domain"])
        type3 = await self.challenge_auth(base64.b64encode(type2).decode("ascii"), encode_to_b64=False)
        if not type3:
            return AuthResult(success=False, handled=False, message="535 5.7.8 invalid NTLM Type 3")

        try:
            parsed = parse_type3(type3)
        except Exception as exc:
            log.warning("bad Type 3: %s", exc)
            return AuthResult(success=False, handled=False, message="535 5.7.8 malformed NTLM Type 3")

        log.info(
            "Type 3 user=%r domain=%r workstation=%r nt_len=%d (tls=%s)",
            parsed["user"], parsed["domain"], parsed["workstation"],
            len(parsed["nt_response"]),
            self.session.ssl is not None,
        )

        if parsed["user"] != CFG["user"]:
            return AuthResult(success=False, handled=False, message="535 5.7.8 authentication failed")
        if ntlmv2_verify(parsed, SERVER_CHALLENGE, CFG["password"]):
            log.info("AUTH NTLM OK")
            return AuthResult(success=True, auth_data=parsed["user"])
        log.info("AUTH NTLM FAILED (wrong password)")
        return AuthResult(success=False, handled=False, message="535 5.7.8 authentication failed")


def plain_authenticator(server, session, envelope, mechanism, auth_data):
    # NOTE: aiosmtpd calls the authenticator synchronously (no await), so this
    # MUST be a regular def — an async def would silently bypass validation.
    log = logging.getLogger("smtp.ntlm")
    try:
        user = auth_data.login.decode() if isinstance(auth_data.login, bytes) else str(auth_data.login)
        pw = auth_data.password.decode() if isinstance(auth_data.password, bytes) else str(auth_data.password)
    except Exception:
        return AuthResult(success=False, handled=False, message="535 5.7.8 bad PLAIN payload")
    if user == CFG["user"] and pw == CFG["password"]:
        log.info("AUTH %s OK (user=%s)", mechanism, user)
        return AuthResult(success=True, auth_data=user)
    log.info("AUTH %s FAILED (user=%s)", mechanism, user)
    return AuthResult(success=False, handled=False, message="535 5.7.8 authentication failed")


class NoopHandler:
    """We never deliver mail; we only care about the auth handshake."""

    async def handle_RCPT(self, server, session, envelope, address, rcpt_options):
        return "250 OK"

    async def handle_DATA(self, server, session, envelope):
        return "250 OK message queued"


def factory():
    tls_ctx = _load_tls_context()
    require_tls = os.environ.get("LEGBA_NTLM_STARTTLS", "").lower() == "require"
    return NtlmSMTP(
        NoopHandler(),
        hostname="mock.legba",
        ident="mock.legba ESMTP NTLM",
        tls_context=tls_ctx,
        require_starttls=require_tls,
        # Default is auth_require_tls=True; flip it off when STARTTLS isn't required.
        auth_require_tls=require_tls,
        authenticator=plain_authenticator,
    )


def main() -> int:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s :: %(message)s",
    )
    port = int(os.environ.get("LEGBA_NTLM_PORT", "2526"))
    starttls_mode = os.environ.get("LEGBA_NTLM_STARTTLS", "off")
    logging.info(
        "mock SMTP NTLM listening on :%d (user=%s domain=%s starttls=%s)",
        port, CFG["user"], CFG["domain"], starttls_mode,
    )
    controller = Controller(handler=NoopHandler(), hostname="0.0.0.0", port=port)
    controller.factory = factory  # override the default SMTP factory
    controller.start()
    try:
        # Block forever.
        import time as _t
        while True:
            _t.sleep(3600)
    except KeyboardInterrupt:
        pass
    controller.stop()
    return 0


if __name__ == "__main__":
    sys.exit(main())
