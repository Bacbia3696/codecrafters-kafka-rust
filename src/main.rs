#![allow(unused_imports)]
use anyhow::Result;
use std::net::SocketAddr;

mod kafka;
mod logging;
mod network;
mod protocol;

use kafka::broker::KafkaBroker;
use logging::{LogUtils, Logger};
use network::server::NetworkServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system
    Logger::init_with_env()?;

    let addr: SocketAddr = "127.0.0.1:9092".parse()?;
    let broker = KafkaBroker::new();
    let server = NetworkServer::new(broker);

    // Log server startup
    LogUtils::log_server_startup(&addr);

    // Start the server
    let result = server.start(addr).await;

    // Log shutdown status
    match &result {
        Ok(_) => LogUtils::log_server_shutdown(true, 0),
        Err(_) => LogUtils::log_server_shutdown(false, 0),
    }

    result
}
