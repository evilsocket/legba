// Raw SMTP NTLM authentication client following [MS-SMTPNTLM].
// Implements the EHLO + (optional STARTTLS) + AUTH NTLM handshake on top of
// the shared `StreamLike` abstraction in utils::net, and reuses the
// ntlmclient crate that already powers http.ntlm1 / http.ntlm2 for the
// Type 1 / Type 2 / Type 3 message generation.

use std::time::Duration;

use base64::prelude::{BASE64_STANDARD, Engine};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};

use crate::creds::Credentials;
use crate::session::Error;
use crate::utils::net::{StreamLike, async_tcp_stream, upgrade_tcp_stream_to_ssl};

/// NTLM version requested by the caller. NTLMv2 matches what modern Exchange
/// installations negotiate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Version {
    V1,
    V2,
}

type Channel = BufStream<Box<dyn StreamLike>>;

/// Read a single SMTP reply (which may span multiple lines for codes like 250).
/// Returns the final response code and the text of the final line including the
/// code prefix.
async fn read_reply(channel: &mut Channel) -> Result<(u16, String), Error> {
    loop {
        let mut line = String::new();
        let n = channel
            .read_line(&mut line)
            .await
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("smtp: connection closed by peer".to_string());
        }
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        if trimmed.len() < 3 {
            return Err(format!("smtp: malformed reply line {:?}", trimmed));
        }
        // trimmed.len() >= 3 is a BYTE length, but trimmed[..3] is a byte-index slice that
        // panics if byte 3 is not a UTF-8 char boundary (a multi-byte reply line from a
        // malicious server). Use get(..3) so a bad boundary is a returned error, not a panic.
        let code: u16 = trimmed
            .get(..3)
            .ok_or_else(|| format!("smtp: malformed reply line {:?}", trimmed))?
            .parse()
            .map_err(|e: std::num::ParseIntError| format!("smtp: bad reply code: {}", e))?;
        // RFC 5321 §4.2: a continuation line uses '-' as the fourth char, the
        // final line uses ' ' (or is exactly 3 chars). We discard intermediates.
        let sep = trimmed.as_bytes().get(3).copied().unwrap_or(b' ');
        if sep == b' ' {
            return Ok((code, trimmed));
        }
    }
}

async fn send_line(channel: &mut Channel, line: &str) -> Result<(), Error> {
    channel
        .write_all(line.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    channel
        .write_all(b"\r\n")
        .await
        .map_err(|e| e.to_string())?;
    channel.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Attempt SMTP authentication against `address` with the supplied credentials,
/// using NTLM (v1 or v2). Returns Ok(true) on a 235 success reply, Ok(false) on
/// any non-235 authentication failure, Err on a protocol-level error.
pub(crate) async fn attempt(
    address: &str,
    creds: &Credentials,
    domain: &str,
    workstation: &str,
    version: Version,
    starttls: bool,
    timeout: Duration,
) -> Result<bool, Error> {
    // The TLS upgrade needs a hostname for SNI. Strip the :port.
    let host = address.rsplit_once(':').map(|(h, _)| h).unwrap_or(address);

    let stream = async_tcp_stream(address, host, timeout, false).await?;
    let mut channel: Channel = BufStream::new(stream);

    // 1. Server banner.
    let (code, msg) = tokio::time::timeout(timeout, read_reply(&mut channel))
        .await
        .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;
    if code != 220 {
        return Err(format!("smtp: unexpected banner: {}", msg));
    }

    let ehlo_host = if workstation.is_empty() {
        "legba"
    } else {
        workstation
    };

    // 2. EHLO.
    send_line(&mut channel, &format!("EHLO {}", ehlo_host)).await?;
    let (code, msg) = tokio::time::timeout(timeout, read_reply(&mut channel))
        .await
        .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;
    if code != 250 {
        return Err(format!("smtp: EHLO rejected: {}", msg));
    }

    // 2b. STARTTLS upgrade + a second EHLO on the secure channel.
    if starttls {
        send_line(&mut channel, "STARTTLS").await?;
        let (code, msg) = tokio::time::timeout(timeout, read_reply(&mut channel))
            .await
            .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;
        if code != 220 {
            return Err(format!("smtp: STARTTLS rejected: {}", msg));
        }

        // Recover the raw stream, upgrade it to TLS, and start a fresh BufStream.
        // BufStream::into_inner() discards any unread buffered bytes; that is
        // fine here because read_reply consumed the full STARTTLS reply.
        let inner = channel.into_inner();
        let tls = upgrade_tcp_stream_to_ssl(inner, host, timeout).await?;
        channel = BufStream::new(tls);

        // Re-EHLO over the TLS channel (RFC 3207).
        send_line(&mut channel, &format!("EHLO {}", ehlo_host)).await?;
        let (code, msg) = tokio::time::timeout(timeout, read_reply(&mut channel))
            .await
            .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;
        if code != 250 {
            return Err(format!("smtp: post-STARTTLS EHLO rejected: {}", msg));
        }
    }

    // 3. AUTH NTLM. Per [MS-SMTPNTLM] §4 the canonical form is the bare
    //    "AUTH NTLM" command, with the Type 1 message sent in the next turn.
    send_line(&mut channel, "AUTH NTLM").await?;
    let (code, msg) = tokio::time::timeout(timeout, read_reply(&mut channel))
        .await
        .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;
    if code != 334 {
        return Err(format!("smtp: AUTH NTLM rejected: {}", msg));
    }

    // 4. Build and send the Type 1 (Negotiate) message. Flags mirror what the
    //    HTTP NTLM client already uses (src/plugins/http/ntlm.rs).
    let nego_flags = ntlmclient::Flags::NEGOTIATE_UNICODE
        | ntlmclient::Flags::REQUEST_TARGET
        | ntlmclient::Flags::NEGOTIATE_NTLM
        | ntlmclient::Flags::NEGOTIATE_WORKSTATION_SUPPLIED;
    let nego_msg = ntlmclient::Message::Negotiate(ntlmclient::NegotiateMessage {
        flags: nego_flags,
        supplied_domain: String::new(),
        supplied_workstation: workstation.to_owned(),
        os_version: Default::default(),
    });
    let nego_b64 = BASE64_STANDARD.encode(nego_msg.to_bytes().map_err(|e| e.to_string())?);
    send_line(&mut channel, &nego_b64).await?;

    // 5. Receive the Type 2 (Challenge) message in a 334 continuation.
    let (code, msg) = tokio::time::timeout(timeout, read_reply(&mut channel))
        .await
        .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;
    if code != 334 {
        return Err(format!("smtp: expected NTLM challenge, got: {}", msg));
    }
    let challenge_b64 = msg.get(4..).unwrap_or("").trim();
    if challenge_b64.is_empty() {
        return Err("smtp: server sent empty NTLM challenge".to_string());
    }
    let challenge_bytes = BASE64_STANDARD
        .decode(challenge_b64)
        .map_err(|e| format!("smtp: bad NTLM challenge base64: {}", e))?;
    let challenge = ntlmclient::Message::try_from(challenge_bytes.as_slice())
        .map_err(|e| format!("smtp: bad NTLM challenge message: {}", e))?;
    let challenge_content = match challenge {
        ntlmclient::Message::Challenge(c) => c,
        other => return Err(format!("smtp: wrong NTLM message type: {:?}", other)),
    };
    let target_info: Vec<u8> = challenge_content
        .target_information
        .iter()
        .flat_map(|ie| ie.to_bytes())
        .collect();

    // 6. Compute and send the Type 3 (Authenticate) response.
    let ntlm_creds = ntlmclient::Credentials {
        username: creds.username.to_owned(),
        password: creds.password.to_owned(),
        domain: domain.to_owned(),
    };
    let response = match version {
        Version::V1 => {
            ntlmclient::respond_challenge_ntlm_v1(challenge_content.challenge, &ntlm_creds)
        }
        Version::V2 => ntlmclient::respond_challenge_ntlm_v2(
            challenge_content.challenge,
            &target_info,
            ntlmclient::get_ntlm_time(),
            &ntlm_creds,
        ),
    };
    let auth_flags = ntlmclient::Flags::NEGOTIATE_UNICODE | ntlmclient::Flags::NEGOTIATE_NTLM;
    let auth_msg = response.to_message(&ntlm_creds, workstation, auth_flags);
    let auth_b64 = BASE64_STANDARD.encode(auth_msg.to_bytes().map_err(|e| e.to_string())?);
    send_line(&mut channel, &auth_b64).await?;

    // 7. Final reply. 235 = authenticated, anything else (typically 535) means
    //    wrong credentials or server-side rejection.
    let (code, _) = tokio::time::timeout(timeout, read_reply(&mut channel))
        .await
        .map_err(|e: tokio::time::error::Elapsed| e.to_string())??;

    // Best-effort QUIT.
    let _ = send_line(&mut channel, "QUIT").await;

    Ok(code == 235)
}
