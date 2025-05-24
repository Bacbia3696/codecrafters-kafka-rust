use crate::protocol::errors::{ProtocolError, ProtocolResult};
use bytes::{Buf, BufMut, BytesMut};

/// Trait for encoding protocol messages to bytes
///
/// This trait provides a common interface for encoding various Kafka protocol
/// structures following the Kafka wire protocol specification.
pub trait ProtocolEncode {
    /// Encodes the message to bytes
    fn encode(&self) -> ProtocolResult<BytesMut>;
}

/// Trait for decoding protocol messages from bytes
///
/// This trait provides a common interface for decoding various Kafka protocol
/// structures from byte buffers.
pub trait ProtocolDecode: Sized {
    /// Decodes the message from a byte buffer
    fn decode(buffer: &mut BytesMut) -> ProtocolResult<Self>;
}

/// Utility functions for Kafka wire protocol encoding/decoding
pub struct WireFormat;

impl WireFormat {
    /// Prints a hex dump of the buffer for debugging
    pub fn debug_hex_dump(buffer: &BytesMut, label: &str) {
        println!("{}: {} bytes", label, buffer.len());
        let bytes: Vec<u8> = buffer.iter().copied().collect();
        for (i, chunk) in bytes.chunks(16).enumerate() {
            print!("{:04X}: ", i * 16);
            for (j, byte) in chunk.iter().enumerate() {
                print!("{:02X} ", byte);
                if j == 7 {
                    print!(" ");
                }
            }
            // Pad if less than 16 bytes
            for _ in chunk.len()..16 {
                print!("   ");
            }
            print!(" |");
            for &byte in chunk {
                if byte.is_ascii_graphic() || byte == b' ' {
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            }
            println!("|");
        }
        println!();
    }

    /// Safely peeks at the next i16 without consuming it
    pub fn peek_i16(buffer: &BytesMut) -> ProtocolResult<i16> {
        if buffer.remaining() < 2 {
            return Err(ProtocolError::insufficient_bytes(2, buffer.remaining()));
        }
        let bytes = &buffer[0..2];
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    /// Safely peeks at the next i32 without consuming it
    pub fn peek_i32(buffer: &BytesMut) -> ProtocolResult<i32> {
        if buffer.remaining() < 4 {
            return Err(ProtocolError::insufficient_bytes(4, buffer.remaining()));
        }
        let bytes = &buffer[0..4];
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Decodes a NULLABLE_STRING from the buffer
    ///
    /// NULLABLE_STRING format:
    /// - Length N as INT16 (i16)
    /// - If N == -1: null value (returns None)
    /// - If N >= 0: N bytes of UTF-8 encoded string (returns Some(String))
    ///
    /// # Examples
    /// ```
    /// use bytes::BytesMut;
    /// use kafka_protocol::encoding::WireFormat;
    ///
    /// let mut buffer = BytesMut::new();
    /// buffer.put_i16(-1); // Null string
    /// let result = WireFormat::decode_nullable_string(&mut buffer).unwrap();
    /// assert_eq!(result, None);
    /// ```
    pub fn decode_nullable_string(buffer: &mut BytesMut) -> ProtocolResult<Option<String>> {
        if buffer.remaining() < 2 {
            return Err(ProtocolError::insufficient_bytes(2, buffer.remaining()));
        }

        let length = buffer.get_i16();

        if length == -1 {
            // Null string
            return Ok(None);
        }

        if length < 0 {
            return Err(ProtocolError::invalid_length(length as i32));
        }

        let length = length as usize;

        if buffer.remaining() < length {
            return Err(ProtocolError::insufficient_bytes(
                length,
                buffer.remaining(),
            ));
        }

        let bytes = buffer.copy_to_bytes(length);
        let string = String::from_utf8(bytes.to_vec())
            .map_err(|e| ProtocolError::InvalidUtf8(e.to_string()))?;

        Ok(Some(string))
    }

    /// Encodes a NULLABLE_STRING to the buffer
    ///
    /// # Examples
    /// ```
    /// use bytes::BytesMut;
    /// use kafka_protocol::encoding::WireFormat;
    ///
    /// let mut buffer = BytesMut::new();
    /// WireFormat::encode_nullable_string(&mut buffer, Some("test")).unwrap();
    /// // buffer now contains: [0x00, 0x04, 't', 'e', 's', 't']
    /// ```
    pub fn encode_nullable_string(
        buffer: &mut BytesMut,
        value: Option<&str>,
    ) -> ProtocolResult<()> {
        match value {
            None => {
                // Null string: length = -1
                buffer.put_i16(-1);
            }
            Some(s) => {
                let bytes = s.as_bytes();
                if bytes.len() > i16::MAX as usize {
                    return Err(ProtocolError::string_too_long(
                        bytes.len(),
                        i16::MAX as usize,
                    ));
                }
                buffer.put_i16(bytes.len() as i16);
                buffer.put_slice(bytes);
            }
        }
        Ok(())
    }

    /// Decodes a regular STRING from the buffer
    ///
    /// STRING format:
    /// - Length N as INT16 (i16)
    /// - N bytes of UTF-8 encoded string
    pub fn decode_string(buffer: &mut BytesMut) -> ProtocolResult<String> {
        if buffer.remaining() < 2 {
            return Err(ProtocolError::insufficient_bytes(2, buffer.remaining()));
        }

        let length = buffer.get_i16();

        if length < 0 {
            return Err(ProtocolError::invalid_length(length as i32));
        }

        let length = length as usize;

        if buffer.remaining() < length {
            return Err(ProtocolError::insufficient_bytes(
                length,
                buffer.remaining(),
            ));
        }

        let bytes = buffer.copy_to_bytes(length);
        let string = String::from_utf8(bytes.to_vec())
            .map_err(|e| ProtocolError::InvalidUtf8(e.to_string()))?;

        Ok(string)
    }

    /// Encodes a regular STRING to the buffer
    pub fn encode_string(buffer: &mut BytesMut, value: &str) -> ProtocolResult<()> {
        let bytes = value.as_bytes();
        if bytes.len() > i16::MAX as usize {
            return Err(ProtocolError::string_too_long(
                bytes.len(),
                i16::MAX as usize,
            ));
        }
        buffer.put_i16(bytes.len() as i16);
        buffer.put_slice(bytes);
        Ok(())
    }

    /// Safely reads an i16 from the buffer with bounds checking
    pub fn decode_i16(buffer: &mut BytesMut) -> ProtocolResult<i16> {
        if buffer.remaining() < 2 {
            return Err(ProtocolError::insufficient_bytes(2, buffer.remaining()));
        }
        Ok(buffer.get_i16())
    }

    /// Safely reads an i32 from the buffer with bounds checking
    pub fn decode_i32(buffer: &mut BytesMut) -> ProtocolResult<i32> {
        if buffer.remaining() < 4 {
            return Err(ProtocolError::insufficient_bytes(4, buffer.remaining()));
        }
        Ok(buffer.get_i32())
    }

    /// Safely reads a u8 from the buffer with bounds checking
    pub fn decode_u8(buffer: &mut BytesMut) -> ProtocolResult<u8> {
        if buffer.remaining() < 1 {
            return Err(ProtocolError::insufficient_bytes(1, buffer.remaining()));
        }
        Ok(buffer.get_u8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_nullable_string_null() {
        let mut buffer = BytesMut::new();
        buffer.put_i16(-1);

        let result = WireFormat::decode_nullable_string(&mut buffer).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_decode_nullable_string_empty() {
        let mut buffer = BytesMut::new();
        buffer.put_i16(0);

        let result = WireFormat::decode_nullable_string(&mut buffer).unwrap();
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn test_decode_nullable_string_valid() {
        let mut buffer = BytesMut::new();
        let test_string = "kafka-client";
        buffer.put_i16(test_string.len() as i16);
        buffer.put_slice(test_string.as_bytes());

        let result = WireFormat::decode_nullable_string(&mut buffer).unwrap();
        assert_eq!(result, Some(test_string.to_string()));
    }

    #[test]
    fn test_encode_nullable_string_roundtrip() {
        let mut buffer = BytesMut::new();
        let test_string = "test-client";

        WireFormat::encode_nullable_string(&mut buffer, Some(test_string)).unwrap();
        let result = WireFormat::decode_nullable_string(&mut buffer).unwrap();

        assert_eq!(result, Some(test_string.to_string()));
    }

    #[test]
    fn test_safe_decode_insufficient_bytes() {
        let mut buffer = BytesMut::new();
        buffer.put_u8(0);

        let result = WireFormat::decode_i16(&mut buffer);
        assert!(matches!(
            result,
            Err(ProtocolError::InsufficientBytes { .. })
        ));
    }

    #[test]
    fn test_peek_functions() {
        let mut buffer = BytesMut::new();
        buffer.put_i16(0x1234);
        buffer.put_i32(0x56789ABC);

        // Test peek doesn't consume bytes
        assert_eq!(WireFormat::peek_i16(&buffer).unwrap(), 0x1234);
        assert_eq!(buffer.len(), 6); // Should still have all 6 bytes

        // Now actually consume and verify
        assert_eq!(WireFormat::decode_i16(&mut buffer).unwrap(), 0x1234);
        assert_eq!(buffer.len(), 4); // Should have 4 bytes left

        assert_eq!(WireFormat::peek_i32(&buffer).unwrap(), 0x56789ABC);
        assert_eq!(WireFormat::decode_i32(&mut buffer).unwrap(), 0x56789ABC);
        assert_eq!(buffer.len(), 0); // Should be empty
    }
}
