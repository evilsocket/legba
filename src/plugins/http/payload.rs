use crate::Credentials;

use super::placeholders;

pub(crate) fn parse_fields(
    payload: Option<&String>,
    creds: &Credentials,
) -> Option<Vec<(String, String)>> {
    if let Some(raw) = payload {
        let mut parsed = vec![];

        for keyval in raw.split('&') {
            let parts: Vec<&str> = keyval.splitn(2, '=').collect();
            let key = parts[0].to_owned();
            let value = placeholders::interpolate(parts[1], creds);

            parsed.push((key, value));
        }

        return Some(parsed);
    }
    None
}

pub(crate) fn parse_body(payload: Option<&String>, creds: &Credentials) -> Option<String> {
    payload.map(|raw| placeholders::interpolate(raw, creds))
}
