use std::time::Duration;

use snmp2::AsyncSession;

use crate::session::{Error, Loot};

use crate::creds::Credentials;

use super::oids;

use snmp2::Oid;

pub(crate) async fn read_from_session(
    session: &mut AsyncSession,
    address: String,
    creds: &Credentials,
    timeout: Duration,
    auth: Option<snmp2::v3::AuthProtocol>,
) -> Result<Option<Vec<Loot>>, Error> {
    // request everything we can
    let seed_oid = Oid::from(&[0, 0]).unwrap();

    while let Ok(res) =
        tokio::time::timeout(timeout, session.getbulk(&[&seed_oid], 0, 0xffff)).await
    {
        match res {
            Ok(response) => {
                let mut data: Vec<(String, String)> = match auth {
                    None => vec![("community".to_owned(), creds.username.to_owned())],
                    Some(proto) => vec![
                        ("auth_proto".to_owned(), format!("{:?}", proto)),
                        ("username".to_owned(), creds.username.to_owned()),
                        ("password".to_owned(), creds.password.to_owned()),
                    ],
                };

                for (oid, val) in response.varbinds {
                    let name = oids::get_oid_name(&oid);

                    // remove the type from the output
                    let mut value = format!("{:?}", val);
                    if let Some((_type, v)) = value.split_once(':') {
                        value = v.trim().to_owned();
                    }

                    data.push((name, value));
                }

                return Ok(Some(vec![Loot::new("snmp", &address, data)]));
            }
            // In case if the engine boot / time counters are not set in the security parameters or
            // they have been changed on the target, e.g. after a reboot, the session returns
            // an error with the AuthUpdated code. In this case, security parameters are automatically
            // updated and the request should be repeated.
            Err(snmp2::Error::AuthUpdated) => continue,
            // session read failure
            Err(e) => {
                if auth.is_none() {
                    // this should not happen for snmp1 and snmp2
                    log::error!("error: {:?}", e);
                }
                break;
            }
        }
    }

    Ok(None)
}
