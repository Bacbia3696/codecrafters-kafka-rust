#![allow(dead_code)]

//! Kafka Protocol Implementation
//!
//! This module provides a comprehensive implementation of the Kafka wire protocol,
//! following SOLID principles and Rust best practices.
//!
//! # Architecture
//!
//! The protocol module is organized into several submodules:
//! - `errors`: Protocol-specific error types and result types
//! - `encoding`: Traits and utilities for encoding/decoding protocol messages
//! - `headers`: Request and response header implementations
//!
//! # Examples
//!
//! ```rust
//! use crate::protocol::{RequestHeaderV2, ResponseHeaderV0, ProtocolEncode};
//!
//! // Create a request header with client ID
//! let request = RequestHeaderV2::with_client_id(1, 2, 42, "my-client");
//!
//! // Encode it to bytes
//! let encoded = request.encode().unwrap();
//!
//! // Create a response header
//! let response = ResponseHeaderV0::new(100, 42);
//! let response_bytes = response.encode().unwrap();
//! ```

pub mod encoding;
pub mod errors;
pub mod headers;

// Re-export commonly used types for convenience
pub use encoding::{ProtocolDecode, ProtocolEncode, WireFormat};
pub use errors::{ProtocolError, ProtocolResult};
pub use headers::{RequestHeaderV2, ResponseHeaderV0};

// Backward compatibility functions for the old protocol.rs interface
use bytes::BytesMut;

/// Decodes a NULLABLE_STRING from the buffer
///
/// This is a backward compatibility function that delegates to WireFormat::decode_nullable_string
pub fn decode_nullable_string(buffer: &mut BytesMut) -> ProtocolResult<Option<String>> {
    WireFormat::decode_nullable_string(buffer)
}

/// Encodes a NULLABLE_STRING to the buffer
///
/// This is a backward compatibility function that delegates to WireFormat::encode_nullable_string
pub fn encode_nullable_string(buffer: &mut BytesMut, value: Option<&str>) -> ProtocolResult<()> {
    WireFormat::encode_nullable_string(buffer, value)
}

/// Protocol specification constants
pub mod spec {
    /// Maximum length for a STRING or NULLABLE_STRING field
    pub const MAX_STRING_LENGTH: usize = i16::MAX as usize;

    /// Null string marker for NULLABLE_STRING fields
    pub const NULL_STRING_MARKER: i16 = -1;

    /// Common API keys used in Kafka protocol
    pub mod api_keys {
        pub const PRODUCE: i16 = 0;
        pub const FETCH: i16 = 1;
        pub const LIST_OFFSETS: i16 = 2;
        pub const METADATA: i16 = 3;
        pub const LEADER_AND_ISR: i16 = 4;
        pub const STOP_REPLICA: i16 = 5;
        pub const UPDATE_METADATA: i16 = 6;
        pub const CONTROLLED_SHUTDOWN: i16 = 7;
        pub const OFFSET_COMMIT: i16 = 8;
        pub const OFFSET_FETCH: i16 = 9;
        pub const FIND_COORDINATOR: i16 = 10;
        pub const JOIN_GROUP: i16 = 11;
        pub const HEARTBEAT: i16 = 12;
        pub const LEAVE_GROUP: i16 = 13;
        pub const SYNC_GROUP: i16 = 14;
        pub const DESCRIBE_GROUPS: i16 = 15;
        pub const LIST_GROUPS: i16 = 16;
        pub const SASL_HANDSHAKE: i16 = 17;
        pub const API_VERSIONS: i16 = 18;
    }

    /// Common error codes used in Kafka protocol
    pub mod error_codes {
        pub const NONE: i16 = 0;
        pub const OFFSET_OUT_OF_RANGE: i16 = 1;
        pub const CORRUPT_MESSAGE: i16 = 2;
        pub const UNKNOWN_TOPIC_OR_PARTITION: i16 = 3;
        pub const INVALID_FETCH_SIZE: i16 = 4;
        pub const LEADER_NOT_AVAILABLE: i16 = 5;
        pub const NOT_LEADER_FOR_PARTITION: i16 = 6;
        pub const REQUEST_TIMED_OUT: i16 = 7;
        pub const BROKER_NOT_AVAILABLE: i16 = 8;
        pub const REPLICA_NOT_AVAILABLE: i16 = 9;
        pub const MESSAGE_TOO_LARGE: i16 = 10;
        pub const STALE_CONTROLLER_EPOCH: i16 = 11;
        pub const OFFSET_METADATA_TOO_LARGE: i16 = 12;
        pub const NETWORK_EXCEPTION: i16 = 13;
        pub const COORDINATOR_LOAD_IN_PROGRESS: i16 = 14;
        pub const COORDINATOR_NOT_AVAILABLE: i16 = 15;
        pub const NOT_COORDINATOR: i16 = 16;
        pub const INVALID_TOPIC_EXCEPTION: i16 = 17;
        pub const RECORD_LIST_TOO_LARGE: i16 = 18;
        pub const NOT_ENOUGH_REPLICAS: i16 = 19;
        pub const NOT_ENOUGH_REPLICAS_AFTER_APPEND: i16 = 20;
        pub const INVALID_REQUIRED_ACKS: i16 = 21;
        pub const ILLEGAL_GENERATION: i16 = 22;
        pub const INCONSISTENT_GROUP_PROTOCOL: i16 = 23;
        pub const INVALID_GROUP_ID: i16 = 24;
        pub const UNKNOWN_MEMBER_ID: i16 = 25;
        pub const INVALID_SESSION_TIMEOUT: i16 = 26;
        pub const REBALANCE_IN_PROGRESS: i16 = 27;
        pub const INVALID_COMMIT_OFFSET_SIZE: i16 = 28;
        pub const TOPIC_AUTHORIZATION_FAILED: i16 = 29;
        pub const GROUP_AUTHORIZATION_FAILED: i16 = 30;
        pub const CLUSTER_AUTHORIZATION_FAILED: i16 = 31;
        pub const INVALID_TIMESTAMP: i16 = 32;
        pub const UNSUPPORTED_SASL_MECHANISM: i16 = 33;
        pub const ILLEGAL_SASL_STATE: i16 = 34;
        pub const UNSUPPORTED_VERSION: i16 = 35;
        pub const TOPIC_ALREADY_EXISTS: i16 = 36;
        pub const INVALID_PARTITIONS: i16 = 37;
        pub const INVALID_REPLICATION_FACTOR: i16 = 38;
        pub const INVALID_REPLICA_ASSIGNMENT: i16 = 39;
        pub const INVALID_CONFIG: i16 = 40;
        pub const NOT_CONTROLLER: i16 = 41;
        pub const INVALID_REQUEST: i16 = 42;
        pub const UNSUPPORTED_FOR_MESSAGE_FORMAT: i16 = 43;
        pub const POLICY_VIOLATION: i16 = 44;
        pub const OUT_OF_ORDER_SEQUENCE_NUMBER: i16 = 45;
        pub const DUPLICATE_SEQUENCE_NUMBER: i16 = 46;
        pub const INVALID_PRODUCER_EPOCH: i16 = 47;
        pub const INVALID_TXN_STATE: i16 = 48;
        pub const INVALID_PRODUCER_ID_MAPPING: i16 = 49;
        pub const INVALID_TRANSACTION_TIMEOUT: i16 = 50;
        pub const CONCURRENT_TRANSACTIONS: i16 = 51;
        pub const TRANSACTION_COORDINATOR_FENCED: i16 = 52;
        pub const TRANSACTIONAL_ID_AUTHORIZATION_FAILED: i16 = 53;
        pub const SECURITY_DISABLED: i16 = 54;
        pub const OPERATION_NOT_ATTEMPTED: i16 = 55;
        pub const KAFKA_STORAGE_ERROR: i16 = 56;
        pub const LOG_DIR_NOT_FOUND: i16 = 57;
        pub const SASL_AUTHENTICATION_FAILED: i16 = 58;
        pub const UNKNOWN_PRODUCER_ID: i16 = 59;
        pub const REASSIGNMENT_IN_PROGRESS: i16 = 60;
        pub const DELEGATION_TOKEN_AUTH_DISABLED: i16 = 61;
        pub const DELEGATION_TOKEN_NOT_FOUND: i16 = 62;
        pub const DELEGATION_TOKEN_OWNER_MISMATCH: i16 = 63;
        pub const DELEGATION_TOKEN_REQUEST_NOT_ALLOWED: i16 = 64;
        pub const DELEGATION_TOKEN_AUTHORIZATION_FAILED: i16 = 65;
        pub const DELEGATION_TOKEN_EXPIRED: i16 = 66;
        pub const INVALID_PRINCIPAL_TYPE: i16 = 67;
        pub const NON_EMPTY_GROUP: i16 = 68;
        pub const GROUP_ID_NOT_FOUND: i16 = 69;
        pub const FETCH_SESSION_ID_NOT_FOUND: i16 = 70;
        pub const INVALID_FETCH_SESSION_EPOCH: i16 = 71;
        pub const LISTENER_NOT_FOUND: i16 = 72;
        pub const TOPIC_DELETION_DISABLED: i16 = 73;
        pub const FENCED_LEADER_EPOCH: i16 = 74;
        pub const UNKNOWN_LEADER_EPOCH: i16 = 75;
        pub const UNSUPPORTED_COMPRESSION_TYPE: i16 = 76;
        pub const STALE_BROKER_EPOCH: i16 = 77;
        pub const OFFSET_NOT_AVAILABLE: i16 = 78;
        pub const MEMBER_ID_REQUIRED: i16 = 79;
        pub const PREFERRED_LEADER_NOT_AVAILABLE: i16 = 80;
        pub const GROUP_MAX_SIZE_REACHED: i16 = 81;
        pub const FENCED_INSTANCE_ID: i16 = 82;
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_protocol_roundtrip() {
        // Test encoding and decoding a complete request/response cycle
        let request = RequestHeaderV2::with_client_id(
            spec::api_keys::METADATA,
            1,
            12345,
            "integration-test-client",
        );

        // Encode the request
        let encoded_request = request.encode().unwrap();

        // Decode it back
        let mut buffer = encoded_request;
        let decoded_request = RequestHeaderV2::decode(&mut buffer).unwrap();

        assert_eq!(request, decoded_request);

        // Create a response with the same correlation ID
        let response = ResponseHeaderV0::new(12345);
        let encoded_response = response.encode().unwrap();

        // Decode the response
        let mut response_buffer = encoded_response;
        let decoded_response = ResponseHeaderV0::decode(&mut response_buffer).unwrap();

        assert_eq!(response, decoded_response);
        assert_eq!(
            decoded_request.correlation_id,
            decoded_response.correlation_id
        );
    }

    #[test]
    fn test_backward_compatibility_functions() {
        // Test that the backward compatibility functions work
        let mut buffer = BytesMut::new();

        // Test encoding
        encode_nullable_string(&mut buffer, Some("test")).unwrap();

        // Test decoding
        let result = decode_nullable_string(&mut buffer).unwrap();
        assert_eq!(result, Some("test".to_string()));
    }
}
