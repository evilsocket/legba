use base64::prelude::{Engine, BASE64_STANDARD};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

use crate::{creds::Credentials, session::Error};

// TODO: test NTLMv1 and NTLMv2 / propagate the set-cookie
pub(crate) async fn handle(
    version: usize,
    url: &str,
    client: Client,
    creds: &Credentials,
    domain: &str,
    workstation: &str,
    headers: HeaderMap<HeaderValue>,
) -> Result<HeaderMap, Error> {
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
    let nego_msg_bytes = nego_msg.to_bytes().map_err(|e| e.to_string())?;
    let nego_b64 = BASE64_STANDARD.encode(&nego_msg_bytes);

    let resp = client
        .get(url)
        .header("Authorization", format!("NTLM {}", nego_b64))
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let challenge_header = if let Some(header) = resp.headers().get("www-authenticate") {
        header
    } else {
        return Err("response missing challenge header".to_string());
    };

    let challenge_b64 = if let Some(challenge) = challenge_header
        .to_str()
        .map_err(|e| e.to_string())?
        .split(' ')
        .nth(1)
    {
        challenge
    } else {
        return Err("second chunk of challenge header missing".to_string());
    };

    let challenge_bytes = BASE64_STANDARD
        .decode(challenge_b64)
        .map_err(|e| e.to_string())?;
    let challenge =
        ntlmclient::Message::try_from(challenge_bytes.as_slice()).map_err(|e| e.to_string())?;
    let challenge_content = match challenge {
        ntlmclient::Message::Challenge(c) => c,
        other => return Err(format!("wrong challenge message: {:?}", other)),
    };
    let target_info_bytes: Vec<u8> = challenge_content
        .target_information
        .iter()
        .flat_map(|ie| ie.to_bytes())
        .collect();

    // calculate the response
    let creds = ntlmclient::Credentials {
        username: creds.username.to_owned(),
        password: creds.password.to_owned(),
        domain: domain.to_owned(),
    };
    let challenge_response = if version == 1 {
        ntlmclient::respond_challenge_ntlm_v1(challenge_content.challenge, &creds)
    } else {
        ntlmclient::respond_challenge_ntlm_v2(
            challenge_content.challenge,
            &target_info_bytes,
            ntlmclient::get_ntlm_time(),
            &creds,
        )
    };

    // assemble the packet and create the header
    let auth_flags = ntlmclient::Flags::NEGOTIATE_UNICODE | ntlmclient::Flags::NEGOTIATE_NTLM;
    let auth_msg = challenge_response.to_message(&creds, workstation, auth_flags);
    let auth_msg_bytes = auth_msg
        .to_bytes()
        .map_err(|e| format!("failed to encode NTLM authentication message: {}", e))?;
    let auth_b64 = BASE64_STANDARD.encode(auth_msg_bytes);
    let mut auth = HeaderMap::new();

    auth.insert(
        "Authorization",
        HeaderValue::from_str(&format!("NTLM {}", auth_b64)).map_err(|e| e.to_string())?,
    );

    Ok(auth)
}
