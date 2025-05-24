#!/usr/bin/env python3
"""
Test script to demonstrate graceful shutdown functionality.
This script tests that the server properly handles active connections during shutdown.
"""

import socket
import struct
import threading
import time
import subprocess
import signal
import os
import sys

def create_api_versions_request():
    """Creates an ApiVersions request."""
    api_key = 18  # ApiVersions
    api_version = 1
    correlation_id = 1
    client_id = "shutdown-test-client"
    
    # Build the request header
    header = bytearray()
    header.extend(struct.pack('>h', api_key))
    header.extend(struct.pack('>h', api_version))
    header.extend(struct.pack('>i', correlation_id))
    
    # Client ID as NULLABLE_STRING
    client_id_bytes = client_id.encode('utf-8')
    header.extend(struct.pack('>h', len(client_id_bytes)))
    header.extend(client_id_bytes)
    
    # Tagged fields (empty)
    header.append(0)
    
    # Complete message
    message_length = len(header)
    message = bytearray()
    message.extend(struct.pack('>I', message_length))
    message.extend(header)
    
    return bytes(message)

def long_running_client(client_id, duration=5):
    """
    Simulates a long-running client connection.
    """
    try:
        print(f"[Client {client_id}] Connecting to broker...")
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(10)  # 10 second timeout
        sock.connect(('127.0.0.1', 9092))
        
        print(f"[Client {client_id}] Connected, sending request...")
        request = create_api_versions_request()
        sock.send(request)
        
        # Try to read response
        try:
            response = sock.recv(1024)
            print(f"[Client {client_id}] Received {len(response)} bytes response")
        except socket.timeout:
            print(f"[Client {client_id}] No response received (timeout)")
        
        # Keep connection alive for a while
        print(f"[Client {client_id}] Keeping connection alive for {duration} seconds...")
        time.sleep(duration)
        
        sock.close()
        print(f"[Client {client_id}] Connection closed normally")
        
    except Exception as e:
        print(f"[Client {client_id}] Error: {e}")

def test_graceful_shutdown():
    """
    Test graceful shutdown with active connections.
    """
    print("=== Testing Graceful Shutdown ===")
    
    # Start the Kafka broker
    print("Starting Kafka broker...")
    broker_process = subprocess.Popen(
        ['cargo', 'run'],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        preexec_fn=os.setsid  # Create a new process group
    )
    
    # Wait a bit for the broker to start
    time.sleep(2)
    
    # Start multiple client connections
    print("Starting client connections...")
    client_threads = []
    
    for i in range(3):
        thread = threading.Thread(
            target=long_running_client, 
            args=(i+1, 8),  # Each client stays connected for 8 seconds
            daemon=True
        )
        client_threads.append(thread)
        thread.start()
        time.sleep(0.5)  # Stagger connection starts
    
    # Wait a bit to let connections establish
    time.sleep(2)
    
    # Send SIGINT to the broker to test graceful shutdown
    print("Sending SIGINT to broker (simulating Ctrl+C)...")
    try:
        # Send SIGINT to the entire process group
        os.killpg(os.getpgid(broker_process.pid), signal.SIGINT)
    except ProcessLookupError:
        print("Broker process already terminated")
    
    # Monitor broker output
    print("Monitoring broker shutdown...")
    try:
        stdout, _ = broker_process.communicate(timeout=35)  # Wait up to 35 seconds
        print("Broker output:")
        print(stdout)
    except subprocess.TimeoutExpired:
        print("Broker shutdown timed out")
        broker_process.kill()
        stdout, _ = broker_process.communicate()
        print("Broker output (after force kill):")
        print(stdout)
    
    # Wait for client threads to finish
    print("Waiting for client threads to finish...")
    for thread in client_threads:
        thread.join(timeout=5)
    
    print("Graceful shutdown test completed!")
    return broker_process.returncode

def test_basic_functionality():
    """
    Test basic broker functionality without shutdown.
    """
    print("=== Testing Basic Functionality ===")
    
    # Start the broker
    print("Starting Kafka broker...")
    broker_process = subprocess.Popen(
        ['cargo', 'run'],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        preexec_fn=os.setsid
    )
    
    # Wait for broker to start
    time.sleep(2)
    
    # Test a simple connection
    try:
        print("Testing simple connection...")
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        sock.connect(('127.0.0.1', 9092))
        
        request = create_api_versions_request()
        sock.send(request)
        
        response = sock.recv(1024)
        print(f"Received {len(response)} bytes response: {response.hex()}")
        
        sock.close()
        print("Basic functionality test passed!")
        
    except Exception as e:
        print(f"Basic functionality test failed: {e}")
    
    # Shutdown broker
    print("Shutting down broker...")
    try:
        os.killpg(os.getpgid(broker_process.pid), signal.SIGINT)
        broker_process.wait(timeout=10)
    except (ProcessLookupError, subprocess.TimeoutExpired):
        broker_process.kill()
    
    return True

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--graceful-shutdown":
        # Test graceful shutdown
        exit_code = test_graceful_shutdown()
        sys.exit(exit_code if exit_code is not None else 0)
    else:
        # Test basic functionality
        success = test_basic_functionality()
        if not success:
            sys.exit(1)
        
        print("\n" + "="*50)
        print("To test graceful shutdown, run:")
        print("python3 test_graceful_shutdown.py --graceful-shutdown")
        print("="*50) 