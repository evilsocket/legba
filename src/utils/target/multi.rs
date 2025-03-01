use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::session::Error;

use cidr_utils::cidr::IpCidr;
use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;

static IPV4_RANGE_PARSER: Lazy<Regex> = lazy_regex!(r"^(\d+)\.(\d+)\.(\d+)\.(\d+)-(\d+):?(\d+)?$");

fn parse_multiple_targets_atom(expression: &str) -> Result<Vec<String>, Error> {
    if let Some(path) = expression.strip_prefix('@') {
        // load from file
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        Ok(reader
            .lines()
            .map(|l| l.unwrap_or_default())
            .filter(|s| !s.is_empty())
            .collect())
    } else if let Some(caps) = IPV4_RANGE_PARSER.captures(expression) {
        // ipv4 range like 192.168.1.1-10 or 192.168.1.1-10:port
        let a: u8 = caps.get(1).unwrap().as_str().parse().unwrap();
        let b: u8 = caps.get(2).unwrap().as_str().parse().unwrap();
        let c: u8 = caps.get(3).unwrap().as_str().parse().unwrap();
        let start: u8 = caps.get(4).unwrap().as_str().parse().unwrap();
        let stop: u8 = caps.get(5).unwrap().as_str().parse().unwrap();

        if stop < start {
            return Err(format!(
                "invalid ip range {}, {} is greater than {}",
                expression, start, stop
            ));
        }

        let port_part = if let Some(port) = caps.get(6) {
            format!(":{}", port.as_str())
        } else {
            "".to_owned()
        };

        let mut range = vec![];
        for d in start..=stop {
            range.push(format!("{}.{}.{}.{}{}", a, b, c, d, port_part));
        }

        Ok(range)
    } else {
        // check for the port part
        let (cidr_part, port_part) = if expression.contains(":[") && expression.ends_with(']') {
            let (cidr, port) = expression.split_once(":[").unwrap();
            (
                cidr,
                if cidr.contains(':') {
                    // ipv6 cidr
                    format!(":[{}", port)
                } else {
                    // ipv4 cidr
                    format!(":{}", port.trim_end_matches(']'))
                },
            )
        } else {
            (expression, "".to_owned())
        };

        // attempt as cidr
        if let Ok(cidr) = IpCidr::from_str(cidr_part) {
            Ok(cidr
                .iter()
                .map(|ip| format!("{}{}", ip, port_part))
                .collect())
        } else {
            // just return as it is
            Ok(vec![expression.to_string()])
        }
    }
}

pub(crate) fn parse_multiple_targets(expression: &str) -> Result<Vec<String>, Error> {
    let mut all = vec![];

    for atom in expression
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        all.extend(parse_multiple_targets_atom(atom)?);
    }

    Ok(all)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::parse_multiple_targets;

    #[test]
    fn can_parse_single() {
        let expected = vec!["127.0.0.1:22".to_owned()];
        let res = parse_multiple_targets("127.0.0.1:22").unwrap();
        assert_eq!(res, expected);

        let expected = vec!["http://www.something.it:8000".to_owned()];
        let res = parse_multiple_targets("http://www.something.it:8000").unwrap();
        assert_eq!(res, expected);

        let expected = vec!["host:1234".to_owned()];
        let res = parse_multiple_targets(",,host:1234,,,").unwrap();
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_from_file() {
        let num_items = 5;
        let tmpdir = tempfile::tempdir().unwrap();
        let tmppath = tmpdir.path().join("targets.txt");
        let mut tmptargets = File::create(&tmppath).unwrap();
        let mut expected = vec![];

        for i in 0..num_items {
            writeln!(tmptargets, "127.0.0.1:{}", i).unwrap();
            expected.push(format!("127.0.0.1:{}", i));
        }
        tmptargets.flush().unwrap();
        drop(tmptargets);

        let res = parse_multiple_targets(&format!("@{}", tmppath.to_str().unwrap())).unwrap();
        assert_eq!(res, expected);
    }

    #[test]
    fn returns_error_for_wrong_filename() {
        let res = parse_multiple_targets("@i-do-not-exist.lol");
        assert!(res.is_err());
    }

    #[test]
    fn can_parse_comma_separated() {
        let expected = Ok(vec![
            "127.0.0.1:22".to_owned(),
            "www.google.com".to_owned(),
            "cnn.com".to_owned(),
            "8.8.8.8:4444".to_owned(),
        ]);
        let res = parse_multiple_targets("127.0.0.1:22, www.google.com, cnn.com,, 8.8.8.8:4444");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_ip_range_without_port() {
        let expected = Ok(vec![
            "192.168.1.1".to_owned(),
            "192.168.1.2".to_owned(),
            "192.168.1.3".to_owned(),
            "192.168.1.4".to_owned(),
            "192.168.1.5".to_owned(),
        ]);
        let res = parse_multiple_targets("192.168.1.1-5");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_ip_range_with_port() {
        let expected = Ok(vec![
            "192.168.1.1:1234".to_owned(),
            "192.168.1.2:1234".to_owned(),
            "192.168.1.3:1234".to_owned(),
            "192.168.1.4:1234".to_owned(),
            "192.168.1.5:1234".to_owned(),
        ]);
        let res = parse_multiple_targets("192.168.1.1-5:1234");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_ipv4_cidr_without_port() {
        let expected = Ok(vec![
            "192.168.1.0".to_owned(),
            "192.168.1.1".to_owned(),
            "192.168.1.2".to_owned(),
            "192.168.1.3".to_owned(),
        ]);
        let res = parse_multiple_targets("192.168.1.0/30");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_ipv4_cidr_with_port() {
        let expected = Ok(vec![
            "192.168.1.0:1234".to_owned(),
            "192.168.1.1:1234".to_owned(),
            "192.168.1.2:1234".to_owned(),
            "192.168.1.3:1234".to_owned(),
        ]);
        let res = parse_multiple_targets("192.168.1.0/30:[1234]");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_ipv6_cidr_without_port() {
        let expected = Ok(vec![
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f0".to_owned(),
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f1".to_owned(),
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f2".to_owned(),
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f3".to_owned(),
        ]);
        let res = parse_multiple_targets("2001:4f8:3:ba:2e0:81ff:fe22:d1f1/126");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_ipv6_cidr_with_port() {
        let expected = Ok(vec![
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f0:[1234]".to_owned(),
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f1:[1234]".to_owned(),
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f2:[1234]".to_owned(),
            "2001:4f8:3:ba:2e0:81ff:fe22:d1f3:[1234]".to_owned(),
        ]);
        let res = parse_multiple_targets("2001:4f8:3:ba:2e0:81ff:fe22:d1f1/126:[1234]");
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_combined() {
        let num_items = 5;
        let tmpdir = tempfile::tempdir().unwrap();
        let tmppath = tmpdir.path().join("targets.txt");
        let mut tmptargets = File::create(&tmppath).unwrap();
        let expected = vec![
            "192.168.1.1",
            "127.0.0.1:0",
            "127.0.0.1:1",
            "127.0.0.1:2",
            "127.0.0.1:3",
            "127.0.0.1:4",
            "8.8.8.8",
            "8.8.8.9",
            "8.8.8.10",
            "8.8.8.11",
        ];

        for i in 0..num_items {
            writeln!(tmptargets, "127.0.0.1:{}", i).unwrap();
        }
        tmptargets.flush().unwrap();
        drop(tmptargets);

        let res = parse_multiple_targets(&format!(
            "192.168.1.1, @{}, 8.8.8.8/30",
            tmppath.to_str().unwrap()
        ))
        .unwrap();
        assert_eq!(res, expected);
    }
}
