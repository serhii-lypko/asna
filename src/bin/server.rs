use tokio::net::TcpListener;
use tokio::signal;

use asna::{DEFAULT_PORT, ingress};

#[tokio::main]
pub async fn main() -> asna::Result<()> {
    let tcp_listener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    ingress::run(tcp_listener, signal::ctrl_c()).await;

    Ok(())
}
