use std::time::Duration;

use crate::session::Error;

pub(crate) trait StreamLike:
    tokio::io::AsyncRead + tokio::io::AsyncWrite + std::fmt::Debug + Send + Sync + Unpin
{
}

impl StreamLike for tokio::net::TcpStream {}

impl StreamLike for async_native_tls::TlsStream<tokio::net::TcpStream> {}

pub(crate) async fn async_tcp_stream(
    address: &str,
    timeout: Duration,
    ssl: bool,
) -> Result<Box<dyn StreamLike>, Error> {
    let tcp_stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(address))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    if ssl {
        let tls = async_native_tls::TlsConnector::new()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true);

        let tls_stream = tokio::time::timeout(timeout, tls.connect("", tcp_stream))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        Ok(Box::new(tls_stream))
    } else {
        Ok(Box::new(tcp_stream))
    }
}
