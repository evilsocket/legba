use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

/// Upper bound on a Kerberos TCP response length prefix. Real KRB-AS/TGS-REP messages are at most
/// a few hundred KB (even with a large PAC); this cap prevents a malicious server's 4-GiB length
/// field from driving an unbounded allocation.
const MAX_KRB_RESPONSE: usize = 16 * 1024 * 1024; // 16 MiB

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug, ValueEnum)]
pub(crate) enum Protocol {
    UDP,
    #[default]
    TCP,
}

pub(crate) fn get(proto: &Protocol, server: SocketAddr) -> Box<dyn Transport> {
    match proto {
        Protocol::UDP => Box::new(UDP::new(server)),
        Protocol::TCP => Box::new(TCP::new(server)),
    }
}

pub trait Transport {
    fn request(&self, timeout: Duration, raw: &[u8]) -> io::Result<Vec<u8>>;
}

#[derive(Debug)]
pub struct UDP {
    server: SocketAddr,
}

impl UDP {
    pub fn new(server: SocketAddr) -> Self {
        Self { server }
    }
}

impl Transport for UDP {
    fn request(&self, _: Duration, raw: &[u8]) -> io::Result<Vec<u8>> {
        // connect and send request
        let sd = UdpSocket::bind("0.0.0.0:0")?;
        sd.connect(self.server)?;
        sd.send(raw)?;

        // compute response size
        let mut resp: Vec<u8> = vec![0; 2048];
        let mut resp_size = sd.peek(&mut resp)?;
        while resp_size == resp.len() {
            resp.append(&mut resp.clone());
            resp_size = sd.peek(&mut resp)?;
        }

        // read response
        let mut raw_response = vec![0; resp_size];
        sd.recv(&mut raw_response)?;

        Ok(raw_response)
    }
}

#[derive(Debug)]
pub struct TCP {
    server: SocketAddr,
}

impl TCP {
    pub fn new(server: SocketAddr) -> Self {
        Self { server }
    }
}

impl Transport for TCP {
    fn request(&self, timeout: Duration, raw: &[u8]) -> io::Result<Vec<u8>> {
        let mut tcp = TcpStream::connect_timeout(&self.server, timeout)?;
        // connect_timeout only bounds the connect; without these a KDC that stalls mid-response
        // would hang the operator's thread indefinitely on read_exact.
        tcp.set_read_timeout(Some(timeout))?;
        tcp.set_write_timeout(Some(timeout))?;

        let req_size = raw.len() as u32;
        let mut req: Vec<u8> = req_size.to_be_bytes().to_vec();
        req.append(&mut raw.to_vec());

        tcp.write_all(&req)?;

        let mut resp_len_raw = [0_u8; 4];
        tcp.read_exact(&mut resp_len_raw)?;
        let resp_len = u32::from_be_bytes(resp_len_raw) as usize;

        // The KDC controls this length prefix; cap it so a malicious/compromised server (or a
        // MITM) cannot drive a multi-GB allocation.
        if resp_len > MAX_KRB_RESPONSE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Kerberos response length {resp_len} exceeds the {MAX_KRB_RESPONSE}-byte limit"),
            ));
        }

        let mut resp: Vec<u8> = vec![0; resp_len];
        tcp.read_exact(&mut resp)?;

        Ok(resp)
    }
}
