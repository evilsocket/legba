use std::time::Duration;

use snmp2::{AsyncSession, Value};

use crate::plugins::snmp::options;
use crate::session::{Error, Loot};

use crate::creds::Credentials;

use super::oids;

use snmp2::Oid;

// https://github.com/roboplc/snmp2/issues/10#issuecomment-3027160417
fn oid_from_str(oidstr: &'_ str) -> Result<Oid<'_>, Error> {
    let oid_vec: Vec<u64> = oidstr
        .split(".")
        .map(|x| x.parse::<u64>().unwrap())
        .collect::<Vec<_>>();

    Oid::from(&oid_vec).map_err(|e| format!("invalid OID: {:?}", e))
}

fn consolidate_results(
    data: Vec<(String, String)>,
    address: String,
) -> Result<Option<Vec<Loot>>, Error> {
    if data.is_empty() {
        Ok(None)
    } else {
        Ok(Some(vec![Loot::new("snmp", &address, data)]))
    }
}

pub(crate) async fn read_from_session(
    options: &options::Options,
    session: &mut AsyncSession,
    address: String,
    creds: &Credentials,
    timeout: Duration,
    auth: Option<snmp2::v3::AuthProtocol>,
) -> Result<Option<Vec<Loot>>, Error> {
    let mut curr_oid = match &options.snmp_oid {
        // single oid, no need to walk
        Some(oid) => oid.clone(),
        // default to the SNMP tree
        None => "1.3.6.1.2.1".to_string(),
    };
    let mut data: Vec<(String, String)> = vec![];
    let mut limit = options.snmp_max;

    // loop for iterating over sequential OIDs
    loop {
        // loop for handling the AuthUpdated
        let mut res = loop {
            // handle timeout
            let res = match tokio::time::timeout(
                timeout,
                session.getnext(&oid_from_str(&curr_oid).unwrap()),
            )
            .await
            {
                Ok(res) => res,
                Err(_) => return consolidate_results(data, address),
            };

            let res = match res {
                Ok(res) => {
                    // initialize authentication data on the first successful request
                    if data.is_empty() {
                        data = match auth {
                            None => {
                                log::info!(
                                    "found community '{}', walking SNMP tree {}...",
                                    creds.username,
                                    if options.snmp_max > 0 {
                                        format!("(max {} OIDs)", options.snmp_max)
                                    } else {
                                        "".to_owned()
                                    }
                                );

                                limit += 1;
                                vec![("community".to_owned(), creds.username.to_owned())]
                            }
                            Some(proto) => {
                                log::info!(
                                    "authenticated with protocol={:?}, username={}, password={}, walking SNMP tree {}...",
                                    proto,
                                    creds.username,
                                    creds.password,
                                    if options.snmp_max > 0 {
                                        format!("(max {} OIDs)", options.snmp_max)
                                    } else {
                                        "".to_owned()
                                    }
                                );

                                limit += 3;
                                vec![
                                    ("auth_proto".to_owned(), format!("{:?}", proto)),
                                    ("username".to_owned(), creds.username.to_owned()),
                                    ("password".to_owned(), creds.password.to_owned()),
                                ]
                            }
                        };
                    }

                    res
                }
                Err(snmp2::Error::AuthUpdated) => {
                    // In case if the engine boot / time counters are not set in the security parameters or
                    // they have been changed on the target, e.g. after a reboot, the session returns
                    // an error with the AuthUpdated code. In this case, security parameters are automatically
                    // updated and the request should be repeated.
                    continue;
                }
                // session read failure
                Err(e) => {
                    if auth.is_none() {
                        // this should not happen for snmp1 and snmp2
                        log::error!("error: {:?}", e);
                    }
                    return consolidate_results(data, address);
                }
            };
            break res;
        };

        // extract value
        if let Some((oid, value)) = res.varbinds.next()
            && !matches!(value, Value::Null)
        {
            let name = oids::get_oid_name(&oid);
            // remove the type from the output
            let mut value = format!("{:?}", value);
            if let Some((_type, v)) = value.split_once(':') {
                value = v.trim().to_owned();
            }

            data.push((name, value));

            // get next oid and clone values that are used outside this scope, to avoid borrows being still in use
            curr_oid = oid.clone().to_string();
        } else {
            break;
        }

        // if we're walking a single oid, or we've reached the limit, break
        if options.snmp_oid.is_some() || (options.snmp_max > 0 && data.len() >= limit) {
            break;
        }
    }

    consolidate_results(data, address)
}
