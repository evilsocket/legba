use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

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
        let mut raw_response = vec![0; resp_size as usize];
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

        let req_size = raw.len() as u32;
        let mut req: Vec<u8> = req_size.to_be_bytes().to_vec();
        req.append(&mut raw.to_vec());

        tcp.write_all(&req)?;

        let mut resp_len_raw = [0_u8; 4];
        tcp.read_exact(&mut resp_len_raw)?;
        let resp_len = u32::from_be_bytes(resp_len_raw);

        let mut resp: Vec<u8> = vec![0; resp_len as usize];
        tcp.read_exact(&mut resp)?;

        Ok(resp)
    }
}
