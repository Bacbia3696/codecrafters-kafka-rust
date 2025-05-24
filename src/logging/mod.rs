use anyhow::Result;
use serde::Serialize;
use std::io;
use tracing::Span;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Configuration for the logging system
///
/// This struct follows the Single Responsibility Principle by focusing
/// solely on logging configuration parameters.
#[derive(Debug, Clone, Serialize)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Whether to log to console
    pub console: bool,
    /// Whether to log to file
    pub file: bool,
    /// Directory for log files
    pub log_dir: String,
    /// Log file prefix
    pub file_prefix: String,
    /// Whether to use JSON format
    pub json_format: bool,
    /// Whether to include timestamps
    pub with_timestamp: bool,
    /// Whether to include thread IDs
    pub with_thread_ids: bool,
    /// Whether to include span information
    pub with_spans: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            console: true,
            file: true,
            log_dir: "./logs".to_string(),
            file_prefix: "kafka-broker".to_string(),
            json_format: false,
            with_timestamp: true,
            with_thread_ids: true,
            with_spans: true,
        }
    }
}

/// Logger component responsible for initializing and managing logging
///
/// This struct follows the Single Responsibility Principle by focusing
/// on logging setup and management. It implements the Open/Closed Principle
/// by allowing extension through configuration without modifying the core logic.
pub struct Logger;

impl Logger {
    /// Initialize the logging system with the given configuration
    ///
    /// This method sets up tracing subscribers for both console and file output
    /// based on the provided configuration.
    pub fn init(config: LogConfig) -> Result<()> {
        // Create the logs directory if it doesn't exist
        if config.file {
            std::fs::create_dir_all(&config.log_dir)?;
        }

        // Create the environment filter
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

        let mut layers = Vec::<Box<dyn Layer<_> + Send + Sync>>::new();

        // Console layer
        if config.console {
            let console_layer = if config.json_format {
                fmt::layer()
                    .json()
                    .with_timer(ChronoUtc::rfc_3339())
                    .with_thread_ids(config.with_thread_ids)
                    .with_span_events(if config.with_spans {
                        fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE
                    } else {
                        fmt::format::FmtSpan::NONE
                    })
                    .with_writer(io::stdout)
                    .boxed()
            } else {
                fmt::layer()
                    .with_timer(ChronoUtc::rfc_3339())
                    .with_thread_ids(config.with_thread_ids)
                    .with_span_events(if config.with_spans {
                        fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE
                    } else {
                        fmt::format::FmtSpan::NONE
                    })
                    .with_writer(io::stdout)
                    .boxed()
            };
            layers.push(console_layer);
        }

        // File layer
        if config.file {
            let file_appender = RollingFileAppender::new(
                Rotation::DAILY,
                &config.log_dir,
                &format!("{}.log", config.file_prefix),
            );

            let file_layer = if config.json_format {
                fmt::layer()
                    .json()
                    .with_timer(ChronoUtc::rfc_3339())
                    .with_thread_ids(config.with_thread_ids)
                    .with_span_events(if config.with_spans {
                        fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE
                    } else {
                        fmt::format::FmtSpan::NONE
                    })
                    .with_writer(file_appender)
                    .boxed()
            } else {
                fmt::layer()
                    .with_timer(ChronoUtc::rfc_3339())
                    .with_thread_ids(config.with_thread_ids)
                    .with_span_events(if config.with_spans {
                        fmt::format::FmtSpan::NEW | fmt::format::FmtSpan::CLOSE
                    } else {
                        fmt::format::FmtSpan::NONE
                    })
                    .with_writer(file_appender)
                    .boxed()
            };
            layers.push(file_layer);
        }

        // Initialize the subscriber
        tracing_subscriber::registry()
            .with(env_filter)
            .with(layers)
            .init();

        tracing::info!(
            config = ?config,
            "Logging system initialized"
        );

        Ok(())
    }

    /// Initialize with default configuration
    pub fn init_default() -> Result<()> {
        Self::init(LogConfig::default())
    }

    /// Initialize with environment-aware defaults
    ///
    /// This method adjusts logging configuration based on environment variables
    /// and deployment context.
    pub fn init_with_env() -> Result<()> {
        let config = LogConfig {
            level: std::env::var("KAFKA_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            console: std::env::var("KAFKA_LOG_CONSOLE")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            file: std::env::var("KAFKA_LOG_FILE")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            log_dir: std::env::var("KAFKA_LOG_DIR").unwrap_or_else(|_| "./logs".to_string()),
            file_prefix: std::env::var("KAFKA_LOG_PREFIX")
                .unwrap_or_else(|_| "kafka-broker".to_string()),
            json_format: std::env::var("KAFKA_LOG_JSON")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
            with_timestamp: true,
            with_thread_ids: true,
            with_spans: true,
        };

        Self::init(config)
    }
}

/// Utility functions for structured logging
pub struct LogUtils;

impl LogUtils {
    /// Create a span for connection handling
    pub fn connection_span(peer_addr: &std::net::SocketAddr) -> Span {
        tracing::info_span!(
            "connection",
            peer_addr = %peer_addr,
            connection_id = tracing::field::Empty,
        )
    }

    /// Create a span for request processing
    pub fn request_span(api_key: u16, correlation_id: i32, client_id: Option<&str>) -> Span {
        tracing::info_span!(
            "request",
            api_key = api_key,
            correlation_id = correlation_id,
            client_id = client_id,
            request_size = tracing::field::Empty,
            response_size = tracing::field::Empty,
        )
    }

    /// Create a span for broker operations
    pub fn broker_span(operation: &str) -> Span {
        tracing::info_span!(
            "broker_operation",
            operation = operation,
            duration_ms = tracing::field::Empty,
        )
    }

    /// Log connection metrics
    pub fn log_connection_metrics(
        peer_addr: &std::net::SocketAddr,
        bytes_read: usize,
        bytes_written: usize,
        duration_ms: u64,
    ) {
        tracing::info!(
            peer_addr = %peer_addr,
            bytes_read = bytes_read,
            bytes_written = bytes_written,
            duration_ms = duration_ms,
            "Connection completed"
        );
    }

    /// Log request metrics
    pub fn log_request_metrics(
        api_key: u16,
        correlation_id: i32,
        request_size: usize,
        response_size: usize,
        processing_time_ms: u64,
        success: bool,
    ) {
        if success {
            tracing::info!(
                api_key = api_key,
                correlation_id = correlation_id,
                request_size = request_size,
                response_size = response_size,
                processing_time_ms = processing_time_ms,
                "Request processed successfully"
            );
        } else {
            tracing::warn!(
                api_key = api_key,
                correlation_id = correlation_id,
                request_size = request_size,
                processing_time_ms = processing_time_ms,
                "Request processing failed"
            );
        }
    }

    /// Log server startup
    pub fn log_server_startup(addr: &std::net::SocketAddr) {
        tracing::info!(
            addr = %addr,
            version = env!("CARGO_PKG_VERSION"),
            "Kafka broker started"
        );
    }

    /// Log server shutdown
    pub fn log_server_shutdown(graceful: bool, active_connections: usize) {
        if graceful {
            tracing::info!(
                active_connections = active_connections,
                "Kafka broker shutdown completed gracefully"
            );
        } else {
            tracing::warn!(
                active_connections = active_connections,
                "Kafka broker shutdown forced"
            );
        }
    }
}

/// Re-export commonly used tracing macros for convenience
pub use tracing::{debug, error, info, trace, warn, Instrument};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_logging() {
        INIT.call_once(|| {
            let config = LogConfig {
                level: "debug".to_string(),
                console: true,
                file: false,
                ..Default::default()
            };
            Logger::init(config).unwrap();
        });
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.console);
        assert!(config.file);
    }

    #[test]
    fn test_connection_span() {
        init_test_logging();
        let addr: std::net::SocketAddr = "127.0.0.1:9092".parse().unwrap();
        let _span = LogUtils::connection_span(&addr);
        // Just test that the span creation works without panicking
    }

    #[test]
    fn test_request_span() {
        init_test_logging();
        let _span = LogUtils::request_span(18, 1, Some("test-client"));
        // Just test that the span creation works without panicking
    }

    #[test]
    fn test_logging_macros() {
        init_test_logging();

        // Test that the macros compile and work
        trace!("This is a trace message");
        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");
    }
}
