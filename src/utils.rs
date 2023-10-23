use crate::session::Error;

pub(crate) fn parse_target(
    target_arg: Option<&String>,
    default_port: u16,
) -> Result<(String, u16), Error> {
    let target = if let Some(target) = target_arg {
        target
    } else {
        return Err("no --target argument specified".to_string());
    };

    let num_colons = target.matches(':').count();

    let (address, port) = if num_colons <= 1 {
        // domain or ipv4
        if let Some((ip, prt)) = target.rsplit_once(':') {
            (
                ip.to_owned(),
                prt.parse::<u16>().map_err(|e| e.to_string())?,
            )
        } else {
            (target.to_owned(), default_port)
        }
    } else {
        // ipv6
        if let Some((ip, prt)) = target.rsplit_once("]:") {
            (
                ip.strip_prefix('[')
                    .ok_or("invalid [ipv6]:port provided".to_string())?
                    .to_owned(),
                prt.parse::<u16>().map_err(|e| e.to_string())?,
            )
        } else {
            (target.to_owned(), default_port)
        }
    };

    Ok((address, port))
}

#[cfg(test)]
mod tests {
    use super::parse_target;

    #[test]
    fn returns_error_if_no_target() {
        let res = parse_target(None, 0);
        assert!(res.is_err());
    }

    #[test]
    fn returns_default_port_if_not_provided_ipv4() {
        let target = Some("127.0.0.1".to_owned());
        let (address, port) = parse_target(target.as_ref(), 4444).unwrap();
        assert_eq!(address, "127.0.0.1");
        assert_eq!(port, 4444);
    }

    #[test]
    fn parses_port_if_provided_ipv4() {
        let target = Some("127.0.0.1:8080".to_owned());
        let (address, port) = parse_target(target.as_ref(), 4444).unwrap();
        assert_eq!(address, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn returns_default_port_if_not_provided_ipv6() {
        let target = Some("::1".to_owned());
        let (address, port) = parse_target(target.as_ref(), 4444).unwrap();
        assert_eq!(address, "::1");
        assert_eq!(port, 4444);
    }

    #[test]
    fn parses_port_if_provided_ipv6() {
        let target = Some("[::1]:8080".to_owned());
        let (address, port) = parse_target(target.as_ref(), 4444).unwrap();
        assert_eq!(address, "::1");
        assert_eq!(port, 8080);
    }
}
