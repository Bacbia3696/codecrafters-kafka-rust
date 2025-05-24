#!/usr/bin/env python3
"""
Simple test client to send Kafka requests to our broker for debugging.
This mimics what a real Kafka client would send.
"""

import socket
import struct

def create_api_versions_request():
    """
    Creates an ApiVersions request (API key 18).
    This is a simple request that Kafka clients typically send first.
    """
    # RequestHeaderV2:
    # - API Key: 18 (ApiVersions)
    # - API Version: 1
    # - Correlation ID: 1
    # - Client ID: "test-client" (nullable string)
    # - Tagged fields: empty (0 byte)
    
    api_key = 18  # ApiVersions
    api_version = 1
    correlation_id = 1
    client_id = "test-client"
    
    # Build the request header
    header = bytearray()
    header.extend(struct.pack('>h', api_key))  # API Key (2 bytes, big-endian)
    header.extend(struct.pack('>h', api_version))  # API Version (2 bytes, big-endian)
    header.extend(struct.pack('>i', correlation_id))  # Correlation ID (4 bytes, big-endian)
    
    # Client ID as NULLABLE_STRING
    client_id_bytes = client_id.encode('utf-8')
    header.extend(struct.pack('>h', len(client_id_bytes)))  # String length (2 bytes, big-endian)
    header.extend(client_id_bytes)  # String content
    
    # Tagged fields (empty for now)
    header.append(0)  # No tagged fields
    
    # For ApiVersions request, there's typically no body after the header
    # The message length should include everything after the length field
    message_length = len(header)
    
    # Build the complete message: [message_length] + [header]
    message = bytearray()
    message.extend(struct.pack('>I', message_length))  # Message length (4 bytes, big-endian)
    message.extend(header)
    
    return bytes(message)

def create_metadata_request():
    """
    Creates a Metadata request (API key 3) with null client_id.
    """
    api_key = 3  # Metadata
    api_version = 1
    correlation_id = 2
    
    # Build the request header
    header = bytearray()
    header.extend(struct.pack('>h', api_key))  # API Key
    header.extend(struct.pack('>h', api_version))  # API Version
    header.extend(struct.pack('>i', correlation_id))  # Correlation ID
    
    # Client ID as NULL NULLABLE_STRING
    header.extend(struct.pack('>h', -1))  # -1 means null
    
    # Tagged fields (empty)
    header.append(0)
    
    # Add a simple metadata request body (empty topic list)
    body = bytearray()
    body.extend(struct.pack('>i', 0))  # Empty topic list (0 topics)
    
    # Complete message
    message_length = len(header) + len(body)
    message = bytearray()
    message.extend(struct.pack('>I', message_length))
    message.extend(header)
    message.extend(body)
    
    return bytes(message)

def test_broker():
    """
    Connect to the broker and send test requests.
    """
    try:
        # Connect to broker
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(('127.0.0.1', 9092))
        
        print("Connected to broker")
        
        # Test 1: Send ApiVersions request with client_id
        print("\n=== Test 1: ApiVersions request with client_id ===")
        request1 = create_api_versions_request()
        print(f"Sending {len(request1)} bytes:")
        print("Hex dump:", request1.hex())
        
        sock.send(request1)
        
        # Try to read response
        try:
            response = sock.recv(1024)
            print(f"Received {len(response)} bytes response")
            if response:
                print("Response hex:", response.hex())
        except socket.timeout:
            print("No response received (timeout)")
        
        # Test 2: Send Metadata request with null client_id
        print("\n=== Test 2: Metadata request with null client_id ===")
        request2 = create_metadata_request()
        print(f"Sending {len(request2)} bytes:")
        print("Hex dump:", request2.hex())
        
        sock.send(request2)
        
        # Try to read response
        try:
            response = sock.recv(1024)
            print(f"Received {len(response)} bytes response")
            if response:
                print("Response hex:", response.hex())
        except socket.timeout:
            print("No response received (timeout)")
        
        sock.close()
        print("\nConnection closed")
        
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    test_broker() 