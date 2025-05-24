use crate::logging::{debug, error, info, warn, LogUtils};
use crate::protocol::{
    ProtocolDecode, ProtocolEncode, RequestHeaderV2, ResponseHeaderV0, WireFormat,
};
use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Core Kafka broker that handles message processing
///
/// This struct encapsulates the main business logic for the Kafka broker,
/// following the Single Responsibility Principle by focusing solely on
/// broker-specific operations.
#[derive(Debug)]
pub struct KafkaBroker {
    // Future: Add fields for topics, partitions, logs, etc.
}

impl KafkaBroker {
    /// Creates a new Kafka broker instance
    pub fn new() -> Self {
        Self {}
    }

    /// Handles incoming client connections
    ///
    /// This method processes client requests and generates appropriate responses.
    /// It follows the Interface Segregation Principle by providing a clean
    /// interface for connection handling.
    pub async fn handle_connection(&self, stream: &mut TcpStream) -> Result<()> {
        let peer_addr = stream.peer_addr()?;
        debug!(peer_addr = %peer_addr, "Starting connection handling");

        loop {
            // Read message length (first 4 bytes)
            let mut length_buffer = [0u8; 4];
            match stream.read_exact(&mut length_buffer).await {
                Ok(_) => {
                    let message_length = u32::from_be_bytes(length_buffer) as usize;
                    debug!(
                        peer_addr = %peer_addr,
                        message_length = message_length,
                        "Read message length prefix"
                    );

                    if message_length == 0 {
                        warn!(peer_addr = %peer_addr, "Received message with zero length");
                        continue;
                    }

                    if message_length > 1024 * 1024 {
                        error!(
                            peer_addr = %peer_addr,
                            message_length = message_length,
                            max_allowed = 1024 * 1024,
                            "Message too large, closing connection"
                        );
                        return Err(anyhow::anyhow!(
                            "Message too large: {} bytes",
                            message_length
                        ));
                    }

                    // Read the message data
                    let mut message_buffer = BytesMut::with_capacity(message_length);
                    message_buffer.resize(message_length, 0);

                    match stream.read_exact(&mut message_buffer).await {
                        Ok(_) => {
                            debug!(
                                peer_addr = %peer_addr,
                                bytes_read = message_length,
                                "Successfully read message data"
                            );

                            // Process the request
                            match self.process_request(&mut message_buffer, peer_addr).await {
                                Ok(response) => {
                                    // Send response length prefix
                                    let response_length = response.len() as u32;
                                    stream.write_all(&response_length.to_be_bytes()).await?;
                                    stream.write_all(&response).await?;

                                    debug!(
                                        peer_addr = %peer_addr,
                                        response_length = response_length,
                                        "Sent response successfully"
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        peer_addr = %peer_addr,
                                        error = %e,
                                        "Failed to process request"
                                    );
                                    // Continue processing other requests instead of closing connection
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                peer_addr = %peer_addr,
                                error = %e,
                                expected_bytes = message_length,
                                "Failed to read message data"
                            );
                            return Err(e.into());
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    info!(peer_addr = %peer_addr, "Client disconnected");
                    break;
                }
                Err(e) => {
                    error!(
                        peer_addr = %peer_addr,
                        error = %e,
                        "Failed to read message length"
                    );
                    return Err(e.into());
                }
            }
        }

        debug!(peer_addr = %peer_addr, "Connection handling completed");
        Ok(())
    }

    /// Processes a single request and returns the response
    async fn process_request(
        &self,
        buffer: &mut BytesMut,
        peer_addr: std::net::SocketAddr,
    ) -> Result<Vec<u8>> {
        let processing_start = Instant::now();
        let original_buffer_len = buffer.len();

        // Parse request header
        let header = match RequestHeaderV2::decode(buffer) {
            Ok(h) => {
                debug!(
                    peer_addr = %peer_addr,
                    api_key = h.request_api_key,
                    api_version = h.request_api_version,
                    correlation_id = h.correlation_id,
                    client_id = ?h.client_id,
                    "Successfully parsed request header"
                );
                h
            }
            Err(e) => {
                error!(
                    peer_addr = %peer_addr,
                    error = %e,
                    buffer_length = original_buffer_len,
                    remaining_bytes = buffer.remaining(),
                    "Failed to parse request header"
                );

                // Log buffer contents for debugging
                if buffer.len() <= 50 {
                    debug!(
                        peer_addr = %peer_addr,
                        buffer_hex = hex::encode(&buffer[..]),
                        "Request buffer contents (hex)"
                    );
                }

                return Err(anyhow::anyhow!("Failed to parse request header: {}", e));
            }
        };

        // Create request span for detailed tracking
        let request_span = LogUtils::request_span(
            header.request_api_key as u16,
            header.correlation_id,
            header.client_id.as_deref(),
        );
        let _span_guard = request_span.enter();

        // Create response header
        let response_header = ResponseHeaderV0 {
            correlation_id: header.correlation_id,
        };

        // Generate response based on API key
        let response_data = match header.request_api_key {
            18 => {
                // ApiVersions request
                debug!("Processing ApiVersions request");
                self.handle_api_versions_request(&header).await?
            }
            _ => {
                warn!(
                    api_key = header.request_api_key,
                    "Unsupported API key, returning error response"
                );
                self.handle_unsupported_request(&header).await?
            }
        };

        // Encode response
        let mut response = BytesMut::new();
        response.extend_from_slice(&response_header.encode()?);
        response.extend_from_slice(&response_data);

        let processing_time = processing_start.elapsed();

        // Log request metrics
        LogUtils::log_request_metrics(
            header.request_api_key as u16,
            header.correlation_id,
            original_buffer_len,
            response.len(),
            processing_time.as_millis() as u64,
            true, // success
        );

        Ok(response.to_vec())
    }

    /// Handles ApiVersions requests
    async fn handle_api_versions_request(&self, _header: &RequestHeaderV2) -> Result<Vec<u8>> {
        debug!("Generating ApiVersions response");

        // Simple ApiVersions response structure:
        // - error_code: i16 = 0 (no error)
        // - api_versions: ARRAY
        //   - api_key: i16
        //   - min_version: i16
        //   - max_version: i16
        // - throttle_time_ms: i32 = 0

        let mut response = BytesMut::new();

        // Error code: 0 (no error)
        response.put_i16(0);

        // API versions array length: 1 (we support ApiVersions only)
        response.put_i32(1);

        // ApiVersions API (key 18)
        response.put_i16(18); // api_key
        response.put_i16(0); // min_version
        response.put_i16(1); // max_version

        // Throttle time: 0
        response.put_i32(0);

        debug!(
            response_length = response.len(),
            "Generated ApiVersions response"
        );
        Ok(response.to_vec())
    }

    /// Handles unsupported requests
    async fn handle_unsupported_request(&self, header: &RequestHeaderV2) -> Result<Vec<u8>> {
        warn!(
            api_key = header.request_api_key,
            "Generating error response for unsupported API"
        );

        // Simple error response with error code 35 (UNSUPPORTED_VERSION)
        let mut response = BytesMut::new();
        response.put_i16(35); // UNSUPPORTED_VERSION error code

        debug!(response_length = response.len(), "Generated error response");
        Ok(response.to_vec())
    }
}

impl Default for KafkaBroker {
    fn default() -> Self {
        Self::new()
    }
}
