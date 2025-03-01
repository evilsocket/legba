use kerberos_asn1::{AsReq, Asn1Object, EncryptedData, PaData, PaEncTsEnc, PrincipalName};
use kerberos_constants::{
    etypes, kdc_options, key_usages::KEY_USAGE_AS_REQ_TIMESTAMP, pa_data_types, principal_names,
};
use rand::{self, Rng};

use crate::creds::Credentials;

// NOTE: copied from kerberos_crypto aes_hmac_sha1::generate_salt, where the realm
// gets uppercased. While this works with Windows domain controllers, it does not
// with Linux based ones.
fn generate_salt(realm: &str, client_name: &str) -> Vec<u8> {
    let mut salt = realm.to_owned();
    let mut lowercase_username = client_name.to_lowercase();

    if lowercase_username.ends_with('$') {
        // client name = "host<client_name>.lower.domain.com"
        salt.push_str("host");
        lowercase_username.pop();
        salt.push_str(&lowercase_username);
        salt.push('.');
        salt.push_str(&realm.to_lowercase());
    } else {
        salt.push_str(&lowercase_username);
    }

    return salt.as_bytes().to_vec();
}

pub(crate) fn create_as_req(realm: &str, creds: &Credentials, for_linux: bool) -> AsReq {
    // create cipher and derive key with salt from user data

    // technically the etype should be negotiated with the DC, but we already know the DC will agree with us ... so ...
    let cipher = kerberos_crypto::new_kerberos_cipher(etypes::AES256_CTS_HMAC_SHA1_96).unwrap();

    let salt = if for_linux {
        // preserve realm's case
        generate_salt(realm, &creds.username)
    } else {
        // make realm uppercase
        cipher.generate_salt(realm, &creds.username)
    };

    let key = cipher.generate_key_from_string(&creds.password, &salt);

    let mut req = kerberos_asn1::AsReq::default();

    req.req_body.kdc_options = kdc_options::RENEWABLE_OK.into();
    req.req_body.realm = realm.to_owned();
    req.req_body.sname = Some(PrincipalName {
        name_type: principal_names::NT_SRV_INST,
        name_string: vec!["krbtgt".to_owned(), realm.to_owned()],
    });
    req.req_body.cname = Some(PrincipalName {
        name_type: principal_names::NT_PRINCIPAL,
        name_string: vec![creds.username.to_owned()],
    });
    req.req_body.till = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::weeks(20 * 52))
        .unwrap()
        .into();
    req.req_body.rtime = Some(
        chrono::Utc::now()
            .checked_add_signed(chrono::Duration::weeks(20 * 52))
            .unwrap()
            .into(),
    );
    req.req_body.nonce = rand::rng().random();
    req.req_body.etypes = vec![cipher.etype()];

    // add pre auth encrypted timestamp
    let timestamp = PaEncTsEnc::from(chrono::Utc::now()).build();
    let encrypted_timestamp = cipher.encrypt(&key, KEY_USAGE_AS_REQ_TIMESTAMP, &timestamp);
    req.padata = Some(vec![PaData::new(
        pa_data_types::PA_ENC_TIMESTAMP,
        EncryptedData::new(cipher.etype(), None, encrypted_timestamp).build(),
    )]);

    req
}
