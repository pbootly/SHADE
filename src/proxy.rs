use crate::config::Config;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

/// Run a TCP proxy that validates connecting IPs and forwards traffic to upstream
pub async fn run_proxy(config_path: &str) -> Result<()> {
    let config = Config::load(config_path)?;
    config.validate()?;

    let listener_addr: SocketAddr = config.proxy.listen_addr.parse()?;
    let upstream_addr: SocketAddr = config.proxy.upstream_addr.parse()?;

    let listener = TcpListener::bind(listener_addr).await?;
    println!(
        "TCP Proxy listening on {}, forwarding to {}",
        listener_addr, upstream_addr
    );

    let storage = crate::server::create_storage(&config).await?;
    let storage = Arc::clone(&storage);

    loop {
        let (mut inbound, addr) = listener.accept().await?;
        let storage = Arc::clone(&storage);
        let upstream_addr = upstream_addr.clone();

        tokio::spawn(async move {
            let client_ip = addr.ip().to_string();

            // Validate connecting IP
            match storage.validate_public_key(&client_ip).await {
                Ok(true) => {
                    println!("Allowed connection from {}", client_ip);
                }
                Ok(false) => {
                    println!("Rejected connection from {}", client_ip);
                    return; // Drop the connection immediately
                }
                Err(e) => {
                    eprintln!("Validation error for {}: {}", client_ip, e);
                    return;
                }
            }

            // Connect to upstream server
            match TcpStream::connect(upstream_addr).await {
                Ok(mut outbound) => {
                    let (mut ri, mut wi) = inbound.split();
                    let (mut ro, mut wo) = outbound.split();

                    // Forward inbound -> outbound
                    let client_to_upstream = tokio::io::copy(&mut ri, &mut wo);
                    // Forward outbound -> inbound
                    let upstream_to_client = tokio::io::copy(&mut ro, &mut wi);

                    if let Err(e) = tokio::try_join!(client_to_upstream, upstream_to_client) {
                        eprintln!("Proxy connection error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to upstream {}: {}", upstream_addr, e);
                }
            }
        });
    }
}
