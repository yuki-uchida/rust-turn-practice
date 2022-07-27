use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::signal;
use turn::server::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "127.0.0.1";
    let port = "3479";
    let conn = Arc::new(UdpSocket::bind(format!("{}:{}", host, port)).await?);
    println!("listening {}...", conn.local_addr()?);
    let server = Server::new(ServerConfig { conn_config: conn }).await?;
    println!("Waiting for Ctrl-C...");
    signal::ctrl_c().await.expect("failed to listen for event");
    println!("\nClosing connection now...");
    server.close().await?;
    println!("closed");
    Ok(())
}
