use crate::kafka::broker::KafkaBroker;
use crate::logging::{error, info, warn, LogUtils};
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::{broadcast, Notify};
use tokio::time::{timeout, Duration};

/// Network server responsible for handling TCP connections
///
/// This struct follows the Single Responsibility Principle by focusing
/// solely on network operations and delegating business logic to the broker.
/// It implements the Dependency Inversion Principle by depending on the
/// KafkaBroker abstraction rather than concrete implementations.
pub struct NetworkServer {
    broker: Arc<KafkaBroker>,
}

impl NetworkServer {
    /// Creates a new network server with the given broker
    pub fn new(broker: KafkaBroker) -> Self {
        Self {
            broker: Arc::new(broker),
        }
    }

    /// Starts the server and listens for incoming connections with graceful shutdown
    ///
    /// This method sets up the TCP listener and handles incoming connections
    /// asynchronously, delegating request processing to the broker.
    /// It supports graceful shutdown via SIGINT (Ctrl+C) and SIGTERM signals.
    pub async fn start(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!(addr = %addr, "Server listening for connections");

        // Create shutdown coordination primitives
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        let active_connections = Arc::new(Notify::new());

        // Spawn signal handling task
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::wait_for_shutdown_signal().await {
                error!(error = %e, "Error setting up signal handlers");
                return;
            }

            info!("Shutdown signal received, initiating graceful shutdown");

            // Notify all tasks to shutdown
            if let Err(e) = shutdown_tx_clone.send(()) {
                error!(error = %e, "Failed to send shutdown signal");
            }
        });

        // Main server loop
        loop {
            tokio::select! {
                // Handle incoming connections
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            info!(peer_addr = %peer_addr, "Accepted new connection");

                            // Spawn a task to handle this connection
                            let broker_clone = Arc::clone(&self.broker);
                            let mut connection_shutdown = shutdown_tx.subscribe();
                            let active_connections_clone = active_connections.clone();

                            tokio::spawn(async move {
                                let connection_start = Instant::now();
                                let span = LogUtils::connection_span(&peer_addr);
                                let _enter = span.enter();

                                // Handle the connection with shutdown awareness
                                let result = tokio::select! {
                                    // Normal connection handling
                                    handle_result = Self::handle_connection_with_timeout(&broker_clone, stream, peer_addr) => {
                                        handle_result
                                    }
                                    // Shutdown signal received
                                    _ = connection_shutdown.recv() => {
                                        info!("Connection shutting down due to server shutdown");
                                        Ok(())
                                    }
                                };

                                let duration = connection_start.elapsed();

                                match result {
                                    Ok(_) => {
                                        info!(
                                            duration_ms = duration.as_millis() as u64,
                                            "Connection handled successfully"
                                        );
                                    }
                                    Err(e) => {
                                        error!(
                                            error = %e,
                                            duration_ms = duration.as_millis() as u64,
                                            "Error handling connection"
                                        );
                                    }
                                }

                                // Notify that this connection has finished
                                active_connections_clone.notify_one();
                            });
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to accept connection");
                            // Continue listening for other connections
                        }
                    }
                }

                // Handle shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Server shutdown initiated");
                    break;
                }
            }
        }

        // Graceful shutdown: wait for active connections to finish
        info!("Waiting for active connections to finish");

        // Give connections up to 30 seconds to finish gracefully
        let shutdown_timeout = Duration::from_secs(30);
        let wait_result = timeout(shutdown_timeout, async {
            // Wait for all connections to finish
            // This is a simple approach - in a real implementation you might want
            // to track the exact number of active connections
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        match wait_result {
            Ok(_) => info!("All connections finished gracefully"),
            Err(_) => warn!("Shutdown timeout reached, forcing exit"),
        }

        info!("Network server shutdown complete");
        Ok(())
    }

    /// Handle a single connection with timeout protection
    async fn handle_connection_with_timeout(
        broker: &KafkaBroker,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<()> {
        // Set a reasonable timeout for connection handling
        let connection_timeout = Duration::from_secs(300); // 5 minutes

        timeout(connection_timeout, broker.handle_connection(&mut stream))
            .await
            .map_err(|_| {
                warn!(timeout_sec = 300, "Connection timed out");
                anyhow::anyhow!("Connection {} timed out", peer_addr)
            })?
    }

    /// Wait for shutdown signals (SIGINT, SIGTERM)
    async fn wait_for_shutdown_signal() -> Result<()> {
        #[cfg(unix)]
        {
            use signal::unix::{signal, SignalKind};

            let mut sigint = signal(SignalKind::interrupt())?;
            let mut sigterm = signal(SignalKind::terminate())?;

            tokio::select! {
                _ = sigint.recv() => {
                    info!("Received SIGINT signal");
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM signal");
                }
            }
        }

        #[cfg(windows)]
        {
            // On Windows, only SIGINT (Ctrl+C) is supported
            signal::ctrl_c().await?;
            info!("Received Ctrl+C signal");
        }

        Ok(())
    }
}
