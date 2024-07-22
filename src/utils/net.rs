use std::time::Duration;

use async_native_tls::TlsStream;

use crate::session::Error;

pub(crate) trait StreamLike:
    tokio::io::AsyncRead + tokio::io::AsyncWrite + std::fmt::Debug + Send + Sync + Unpin
{
}

impl StreamLike for tokio::net::TcpStream {}

impl StreamLike for async_native_tls::TlsStream<tokio::net::TcpStream> {}
impl StreamLike for async_native_tls::TlsStream<Box<dyn StreamLike>> {}

pub(crate) async fn upgrade_tcp_stream_to_tls(
    tcp_stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Result<TlsStream<Box<dyn StreamLike>>, Error> {
    let tls = async_native_tls::TlsConnector::new()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true);

    tokio::time::timeout(timeout, tls.connect("", tcp_stream))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

pub(crate) async fn upgrade_tcp_stream_to_ssl(
    tcp_stream: Box<dyn StreamLike>,
    timeout: Duration,
) -> Result<Box<dyn StreamLike>, Error> {
    let tls_stream = upgrade_tcp_stream_to_tls(tcp_stream, timeout).await?;

    Ok(Box::new(tls_stream))
}

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
        upgrade_tcp_stream_to_ssl(Box::new(tcp_stream), timeout).await
    } else {
        Ok(Box::new(tcp_stream))
    }
}
