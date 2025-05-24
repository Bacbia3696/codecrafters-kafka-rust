use thiserror::Error;

/// Protocol-specific error types for Kafka wire protocol operations
///
/// This enum provides comprehensive error handling for all protocol-related
/// operations, following Rust best practices for error handling.
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Insufficient bytes in buffer: expected {expected}, got {actual}")]
    InsufficientBytes { expected: usize, actual: usize },

    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8(String),

    #[error("String too long: {length} bytes exceeds maximum {max}")]
    StringTooLong { length: usize, max: usize },

    #[error("Invalid length field: {length}")]
    InvalidLength { length: i32 },

    #[error("Buffer overflow: attempted to read {attempted} bytes from {available}")]
    BufferOverflow { attempted: usize, available: usize },
}

/// Type alias for protocol operation results
pub type ProtocolResult<T> = Result<T, ProtocolError>;

impl ProtocolError {
    /// Creates an insufficient bytes error with context
    pub fn insufficient_bytes(expected: usize, actual: usize) -> Self {
        Self::InsufficientBytes { expected, actual }
    }

    /// Creates a buffer overflow error with context
    pub fn buffer_overflow(attempted: usize, available: usize) -> Self {
        Self::BufferOverflow {
            attempted,
            available,
        }
    }

    /// Creates a string too long error with context
    pub fn string_too_long(length: usize, max: usize) -> Self {
        Self::StringTooLong { length, max }
    }

    /// Creates an invalid length error
    pub fn invalid_length(length: i32) -> Self {
        Self::InvalidLength { length }
    }
}
