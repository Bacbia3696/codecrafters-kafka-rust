use crate::protocol::encoding::{ProtocolDecode, ProtocolEncode, WireFormat};
use crate::protocol::errors::{ProtocolError, ProtocolResult};
use bytes::{Buf, BufMut, BytesMut};

/// Kafka Response Header Version 0
///
/// This represents the header structure for Kafka protocol responses.
/// It follows the Kafka wire protocol specification.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseHeaderV0 {
    pub correlation_id: i32,
}

impl ResponseHeaderV0 {
    /// Creates a new response header
    pub fn new(correlation_id: i32) -> Self {
        Self { correlation_id }
    }
}

impl ProtocolEncode for ResponseHeaderV0 {
    /// Encodes the header to bytes following Kafka wire protocol
    fn encode(&self) -> ProtocolResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(4); // 4 bytes per i32 field

        buffer.put_i32(self.correlation_id);

        Ok(buffer)
    }
}

impl ProtocolDecode for ResponseHeaderV0 {
    fn decode(buffer: &mut BytesMut) -> ProtocolResult<Self> {
        let correlation_id = WireFormat::decode_i32(buffer)?;

        Ok(Self { correlation_id })
    }
}

/// Kafka Request Header Version 2
///
/// This represents the header structure for Kafka protocol requests version 2.
/// It includes support for nullable client_id and tagged fields.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestHeaderV2 {
    pub request_api_key: i16,
    pub request_api_version: i16,
    pub correlation_id: i32,
    pub client_id: Option<String>, // NULLABLE_STRING
                                   // Tagged fields are currently not fully implemented
}

impl RequestHeaderV2 {
    /// Creates a new request header
    pub fn new(
        request_api_key: i16,
        request_api_version: i16,
        correlation_id: i32,
        client_id: Option<String>,
    ) -> Self {
        Self {
            request_api_key,
            request_api_version,
            correlation_id,
            client_id,
        }
    }

    /// Convenience method to create a header with a client ID
    pub fn with_client_id(
        request_api_key: i16,
        request_api_version: i16,
        correlation_id: i32,
        client_id: impl Into<String>,
    ) -> Self {
        Self::new(
            request_api_key,
            request_api_version,
            correlation_id,
            Some(client_id.into()),
        )
    }

    /// Convenience method to create a header without a client ID
    pub fn without_client_id(
        request_api_key: i16,
        request_api_version: i16,
        correlation_id: i32,
    ) -> Self {
        Self::new(request_api_key, request_api_version, correlation_id, None)
    }
}

impl ProtocolEncode for RequestHeaderV2 {
    fn encode(&self) -> ProtocolResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_i16(self.request_api_key);
        buffer.put_i16(self.request_api_version);
        buffer.put_i32(self.correlation_id);

        WireFormat::encode_nullable_string(&mut buffer, self.client_id.as_deref())?;

        // Empty tag section (tagged fields not implemented yet)
        buffer.put_u8(0);

        Ok(buffer)
    }
}

impl ProtocolDecode for RequestHeaderV2 {
    fn decode(buffer: &mut BytesMut) -> ProtocolResult<Self> {
        // Ensure we have at least the minimum required bytes for the fixed fields
        if buffer.remaining() < 8 {
            return Err(ProtocolError::insufficient_bytes(8, buffer.remaining()));
        }

        let request_api_key = WireFormat::decode_i16(buffer)?;
        let request_api_version = WireFormat::decode_i16(buffer)?;
        let correlation_id = WireFormat::decode_i32(buffer)?;

        // Decode the nullable client_id
        let client_id = WireFormat::decode_nullable_string(buffer)?;

        // Skip tagged fields for now (assuming empty tag section with 0 length)
        if buffer.remaining() >= 1 {
            let _tagged_fields = WireFormat::decode_u8(buffer)?; // Usually 0 for no tags
        }

        Ok(Self {
            request_api_key,
            request_api_version,
            correlation_id,
            client_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_header_v0_creation() {
        let header = ResponseHeaderV0::new(42);
        assert_eq!(header.correlation_id, 42);
    }

    #[test]
    fn test_response_header_v0_encoding() {
        let header = ResponseHeaderV0::new(7);
        let encoded = header.encode().unwrap();

        // Should be 4 bytes total (4 for correlation_id)
        assert_eq!(encoded.len(), 4);

        // Convert to bytes for verification
        let bytes = encoded.as_ref();

        // Check correlation_id (7 in big-endian)
        assert_eq!(&bytes[4..8], &[0, 0, 0, 7]);
    }

    #[test]
    fn test_response_header_v0_roundtrip() {
        let original = ResponseHeaderV0::new(42);
        let encoded = original.encode().unwrap();
        let mut buffer = encoded;
        let decoded = ResponseHeaderV0::decode(&mut buffer).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_request_header_v2_with_client_id() {
        let header = RequestHeaderV2::with_client_id(1, 2, 42, "test-client");
        assert_eq!(header.request_api_key, 1);
        assert_eq!(header.request_api_version, 2);
        assert_eq!(header.correlation_id, 42);
        assert_eq!(header.client_id, Some("test-client".to_string()));
    }

    #[test]
    fn test_request_header_v2_without_client_id() {
        let header = RequestHeaderV2::without_client_id(1, 2, 42);
        assert_eq!(header.request_api_key, 1);
        assert_eq!(header.request_api_version, 2);
        assert_eq!(header.correlation_id, 42);
        assert_eq!(header.client_id, None);
    }

    #[test]
    fn test_request_header_v2_roundtrip_with_client_id() {
        let original = RequestHeaderV2::with_client_id(1, 2, 42, "test-client");
        let encoded = original.encode().unwrap();
        let mut buffer = encoded;
        let decoded = RequestHeaderV2::decode(&mut buffer).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_request_header_v2_roundtrip_with_null_client_id() {
        let original = RequestHeaderV2::without_client_id(1, 2, 42);
        let encoded = original.encode().unwrap();
        let mut buffer = encoded;
        let decoded = RequestHeaderV2::decode(&mut buffer).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_request_header_v2_insufficient_bytes() {
        let mut buffer = BytesMut::new();
        buffer.put_i32(42); // Only 4 bytes, but we need at least 8

        let result = RequestHeaderV2::decode(&mut buffer);
        assert!(matches!(
            result,
            Err(ProtocolError::InsufficientBytes { .. })
        ));
    }
}
