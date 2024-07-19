use crate::creds::Credentials;

const USERNAME: &str = "{USERNAME}";
const PASSWORD: &str = "{PASSWORD}";
const PAYLOAD: &str = "{PAYLOAD}";

pub(crate) fn interpolate(data: &str, creds: &Credentials) -> String {
    let mut parsed = data.to_owned();

    // undo query encoding of interpolation params
    for placeholder in [USERNAME, PASSWORD, PAYLOAD] {
        let encoded_lwr = placeholder.replace('{', "%7b").replace('}', "%7d");
        let encoded_upr = placeholder.replace('{', "%7B").replace('}', "%7D");

        parsed = parsed
            .replace(&encoded_lwr, placeholder)
            .replace(&encoded_upr, placeholder);
    }

    // interpolate placeholders
    parsed
        .replace(USERNAME, &creds.username)
        .replace(PAYLOAD, &creds.username)
        .replace(PASSWORD, &creds.password)
}
