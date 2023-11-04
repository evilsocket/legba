use crate::session::Error;

pub(crate) fn parse_target(target: &str, default_port: u16) -> Result<(String, u16), Error> {
    if target.contains(' ') || target.contains(',') {
        return Err(format!(
            "'{}' is not a valid target, maybe you meant to use --multiple instead of --target?",
            target
        ));
    }

    // remove <proto>:// if present
    let target = if target.contains("://") {
        target.split_once("://").unwrap().1
    } else {
        target
    };

    // remove /<whatever> if present
    let target = if target.contains('/') {
        target.split_once('/').unwrap().0
    } else {
        target
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

#[inline]
pub(crate) fn parse_target_address(target: &str, default_port: u16) -> Result<String, Error> {
    let (host, port) = parse_target(target, default_port)?;
    Ok(format!("{}:{}", host, port))
}

#[cfg(test)]
mod tests {
    use super::parse_target;

    #[test]
    fn returns_default_port_if_not_provided_ipv4() {
        let (address, port) = parse_target("127.0.0.1", 4444).unwrap();
        assert_eq!(address, "127.0.0.1");
        assert_eq!(port, 4444);
    }

    #[test]
    fn parses_port_if_provided_ipv4() {
        let (address, port) = parse_target("127.0.0.1:8080", 4444).unwrap();
        assert_eq!(address, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn returns_default_port_if_not_provided_ipv6() {
        let (address, port) = parse_target("::1", 4444).unwrap();
        assert_eq!(address, "::1");
        assert_eq!(port, 4444);
    }

    #[test]
    fn parses_port_if_provided_ipv6() {
        let (address, port) = parse_target("[::1]:8080", 4444).unwrap();
        assert_eq!(address, "::1");
        assert_eq!(port, 8080);
    }
}
