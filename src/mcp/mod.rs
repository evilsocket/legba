use std::net::SocketAddr;

use rmcp::transport::sse_server::SseServer;

use crate::mcp::service::Service;
use crate::Options;
use crate::session::Error;

mod service;

pub(crate) async fn start(opts: Options) -> Result<(), Error> {
    let address = opts.mcp.unwrap();
    if !address.contains(':') {
        return Err("no port specified, please specify a port in the format host:port".to_string());
    }

    log::info!("starting mcp server on http://{}/sse ...", &address);

    if !address.contains("localhost") && !address.contains("127.0.0.1") {
        log::warn!(
            "this server does not provide any authentication and you are binding it to an external address, use with caution!"
        );
    }

    let address: SocketAddr = address.parse().map_err(|e: std::net::AddrParseError| e.to_string())?;
    let concurrency = opts.concurrency;
    let create_service_fn = move || Service::new(concurrency);
    let ct = SseServer::serve(address)
        .await.map_err(|e| e.to_string())?
        .with_service(create_service_fn);

    tokio::signal::ctrl_c().await.map_err(|e| e.to_string())?;
    ct.cancel();
    Ok(())
}
