use crate::Credentials;

const USERNAME_PLACEHOLDER: &str = "{USERNAME}";
const PASSWORD_PLACEHOLDER: &str = "{PASSWORD}";

pub(crate) fn parse_fields(
    payload: Option<&String>,
    creds: &Credentials,
) -> Option<Vec<(String, String)>> {
    if let Some(raw) = payload {
        let mut parsed = vec![];

        for keyval in raw.split('&') {
            let parts: Vec<&str> = keyval.splitn(2, '=').collect();
            let key = parts[0].to_owned();
            let value = match parts[1] {
                USERNAME_PLACEHOLDER => creds.username.to_owned(),
                PASSWORD_PLACEHOLDER => creds.password.to_owned(),
                _ => parts[1].to_owned(),
            };
            parsed.push((key, value));
        }

        return Some(parsed);
    }
    None
}

pub(crate) fn parse_body(payload: Option<&String>, creds: &Credentials) -> Option<String> {
    if let Some(raw) = payload {
        return Some(
            raw.replace(USERNAME_PLACEHOLDER, &creds.username)
                .replace(PASSWORD_PLACEHOLDER, &creds.password),
        );
    }
    None
}
