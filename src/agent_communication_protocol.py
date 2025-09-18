#!/usr/bin/env python3
"""
Agent Communication Protocol - Advanced inter-agent communication and coordination
"""

import asyncio
import json
import uuid
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Callable
import logging
from dataclasses import dataclass, asdict
from enum import Enum
import threading
import queue
import hashlib
import hmac
import base64

# Advanced messaging
import zmq
import redis
from kafka import KafkaProducer, KafkaConsumer
import websockets
import aiohttp

class ProtocolType(Enum):
    REQUEST_RESPONSE = "request_response"
    PUBLISH_SUBSCRIBE = "publish_subscribe"
    STREAMING = "streaming"
    BROADCAST = "broadcast"
    MULTICAST = "multicast"

class MessagePriority(Enum):
    LOW = 1
    NORMAL = 2
    HIGH = 3
    CRITICAL = 4
    EMERGENCY = 5

@dataclass
class CommunicationMessage:
    """Advanced communication message structure"""
    message_id: str
    sender_id: str
    receiver_id: Optional[str]  # None for broadcast
    protocol_type: ProtocolType
    message_type: str
    content: Dict[str, Any]
    timestamp: datetime
    priority: MessagePriority
    ttl: int  # Time to live in seconds
    correlation_id: Optional[str] = None
    reply_to: Optional[str] = None
    signature: Optional[str] = None
    encryption_key: Optional[str] = None
    metadata: Dict[str, Any] = None

@dataclass
class AgentCapability:
    """Agent capability definition"""
    capability_type: str
    description: str
    parameters: Dict[str, Any]
    performance_metrics: Dict[str, float]

class AgentCommunicationProtocol:
    """Advanced communication protocol for AI agents"""
    
    def __init__(self, agent_id: str, config: Dict[str, Any]):
        self.agent_id = agent_id
        self.config = config
        self.message_handlers = {}
        self.subscriptions = {}
        self.message_queue = asyncio.Queue()
        self.response_waiters = {}
        self.capabilities = {}
        self.peer_registry = {}
        
        # Communication channels
        self.zmq_context = zmq.Context()
        self.redis_client = redis.Redis(host='localhost', port=6379, db=0)
        self.kafka_producer = KafkaProducer(
            bootstrap_servers=['localhost:9092'],
            value_serializer=lambda v: json.dumps(v).encode('utf-8')
        )
        self.kafka_consumer = None
        self.websocket_server = None
        self.websocket_clients = {}
        
        # Security
        self.private_key = None
        self.public_key = None
        self.shared_secrets = {}
        
        self.initialize_protocol()
    
    def initialize_protocol(self):
        """Initialize communication protocol"""
        try:
            # Initialize ZMQ sockets
            self.initialize_zmq()
            
            # Initialize Kafka
            self.initialize_kafka()
            
            # Initialize WebSocket server
            self.initialize_websocket()
            
            # Initialize security
            self.initialize_security()
            
            logging.info(f"Communication protocol initialized for agent {self.agent_id}")
            
        except Exception as e:
            logging.error(f"Failed to initialize communication protocol: {e}")
            raise
    
    def initialize_zmq(self):
        """Initialize ZMQ communication"""
        try:
            # Request-Response socket
            self.req_socket = self.zmq_context.socket(zmq.REQ)
            self.req_socket.bind(f"tcp://*:{5550 + hash(self.agent_id) % 1000}")
            
            # Publish-Subscribe socket
            self.pub_socket = self.zmq_context.socket(zmq.PUB)
            self.pub_socket.bind(f"tcp://*:{5551 + hash(self.agent_id) % 1000}")
            
            # Subscribe socket
            self.sub_socket = self.zmq_context.socket(zmq.SUB)
            self.sub_socket.setsockopt(zmq.SUBSCRIBE, b"")
            
            # Dealer socket for advanced messaging
            self.dealer_socket = self.zmq_context.socket(zmq.DEALER)
            self.dealer_socket.bind(f"tcp://*:{5552 + hash(self.agent_id) % 1000}")
            
        except Exception as e:
            logging.error(f"Failed to initialize ZMQ: {e}")
            raise
    
    def initialize_kafka(self):
        """Initialize Kafka communication"""
        try:
            self.kafka_consumer = KafkaConsumer(
                f"agent_{self.agent_id}",
                bootstrap_servers=['localhost:9092'],
                value_deserializer=lambda m: json.loads(m.decode('utf-8')),
                group_id=f"agent_group_{self.agent_id}",
                auto_offset_reset='latest'
            )
            
        except Exception as e:
            logging.error(f"Failed to initialize Kafka: {e}")
            raise
    
    def initialize_websocket(self):
        """Initialize WebSocket communication"""
        try:
            # WebSocket server for real-time communication
            self.websocket_port = 8000 + hash(self.agent_id) % 1000
            
        except Exception as e:
            logging.error(f"Failed to initialize WebSocket: {e}")
            raise
    
    def initialize_security(self):
        """Initialize security components"""
        try:
            # Generate key pair for digital signatures
            from cryptography.hazmat.primitives.asymmetric import rsa
            from cryptography.hazmat.primitives import serialization
            
            self.private_key = rsa.generate_private_key(
                public_exponent=65537,
                key_size=2048
            )
            self.public_key = self.private_key.public_key()
            
            # Generate shared secrets for encryption
            self.generate_shared_secrets()
            
        except Exception as e:
            logging.error(f"Failed to initialize security: {e}")
            raise
    
    def generate_shared_secrets(self):
        """Generate shared secrets with other agents"""
        try:
            # This would generate shared secrets with known agents
            # For now, using placeholder
            self.shared_secrets = {
                'default': 'shared_secret_key_placeholder'
            }
            
        except Exception as e:
            logging.error(f"Failed to generate shared secrets: {e}")
    
    async def start_protocol(self):
        """Start the communication protocol"""
        try:
            # Start message handling
            asyncio.create_task(self.handle_messages())
            
            # Start Kafka consumer
            asyncio.create_task(self.consume_kafka_messages())
            
            # Start WebSocket server
            asyncio.create_task(self.start_websocket_server())
            
            # Start peer discovery
            asyncio.create_task(self.discover_peers())
            
            logging.info(f"Communication protocol started for agent {self.agent_id}")
            
        except Exception as e:
            logging.error(f"Failed to start communication protocol: {e}")
            raise
    
    async def stop_protocol(self):
        """Stop the communication protocol"""
        try:
            # Close all sockets
            self.req_socket.close()
            self.pub_socket.close()
            self.sub_socket.close()
            self.dealer_socket.close()
            
            # Close Kafka consumer
            if self.kafka_consumer:
                self.kafka_consumer.close()
            
            # Close WebSocket server
            if self.websocket_server:
                self.websocket_server.close()
            
            logging.info(f"Communication protocol stopped for agent {self.agent_id}")
            
        except Exception as e:
            logging.error(f"Failed to stop communication protocol: {e}")
    
    async def send_message(self, receiver_id: str, message_type: str, 
                          content: Dict[str, Any], protocol_type: ProtocolType = ProtocolType.REQUEST_RESPONSE,
                          priority: MessagePriority = MessagePriority.NORMAL,
                          ttl: int = 3600, correlation_id: Optional[str] = None) -> str:
        """Send message to another agent"""
        try:
            message = CommunicationMessage(
                message_id=str(uuid.uuid4()),
                sender_id=self.agent_id,
                receiver_id=receiver_id,
                protocol_type=protocol_type,
                message_type=message_type,
                content=content,
                timestamp=datetime.now(),
                priority=priority,
                ttl=ttl,
                correlation_id=correlation_id
            )
            
            # Sign message
            message.signature = self.sign_message(message)
            
            # Encrypt message if needed
            if receiver_id in self.shared_secrets:
                message = self.encrypt_message(message, receiver_id)
            
            # Send based on protocol type
            if protocol_type == ProtocolType.REQUEST_RESPONSE:
                await self.send_request_response(message)
            elif protocol_type == ProtocolType.PUBLISH_SUBSCRIBE:
                await self.send_publish_subscribe(message)
            elif protocol_type == ProtocolType.STREAMING:
                await self.send_streaming(message)
            elif protocol_type == ProtocolType.BROADCAST:
                await self.send_broadcast(message)
            elif protocol_type == ProtocolType.MULTICAST:
                await self.send_multicast(message)
            
            logging.info(f"Message sent from {self.agent_id} to {receiver_id}")
            return message.message_id
            
        except Exception as e:
            logging.error(f"Failed to send message: {e}")
            raise
    
    async def send_request_response(self, message: CommunicationMessage):
        """Send request-response message"""
        try:
            # Send via ZMQ REQ socket
            self.req_socket.send_string(json.dumps(asdict(message)))
            
            # Wait for response
            response_data = self.req_socket.recv_string()
            response = CommunicationMessage(**json.loads(response_data))
            
            # Handle response
            await self.handle_response(response)
            
        except Exception as e:
            logging.error(f"Failed to send request-response: {e}")
    
    async def send_publish_subscribe(self, message: CommunicationMessage):
        """Send publish-subscribe message"""
        try:
            # Send via ZMQ PUB socket
            topic = f"{message.receiver_id}.{message.message_type}"
            self.pub_socket.send_string(f"{topic} {json.dumps(asdict(message))}")
            
        except Exception as e:
            logging.error(f"Failed to send publish-subscribe: {e}")
    
    async def send_streaming(self, message: CommunicationMessage):
        """Send streaming message"""
        try:
            # Send via Kafka for streaming
            self.kafka_producer.send(
                f"stream_{message.receiver_id}",
                value=asdict(message)
            )
            
        except Exception as e:
            logging.error(f"Failed to send streaming: {e}")
    
    async def send_broadcast(self, message: CommunicationMessage):
        """Send broadcast message"""
        try:
            # Send to all known peers
            for peer_id in self.peer_registry:
                message.receiver_id = peer_id
                await self.send_publish_subscribe(message)
            
        except Exception as e:
            logging.error(f"Failed to send broadcast: {e}")
    
    async def send_multicast(self, message: CommunicationMessage):
        """Send multicast message"""
        try:
            # Send to specific group of agents
            multicast_group = message.content.get('multicast_group', [])
            
            for peer_id in multicast_group:
                message.receiver_id = peer_id
                await self.send_publish_subscribe(message)
            
        except Exception as e:
            logging.error(f"Failed to send multicast: {e}")
    
    async def handle_messages(self):
        """Handle incoming messages"""
        while True:
            try:
                # Check for messages in queue
                if not self.message_queue.empty():
                    message = await self.message_queue.get()
                    await self.process_message(message)
                
                await asyncio.sleep(0.1)
                
            except Exception as e:
                logging.error(f"Error handling messages: {e}")
                await asyncio.sleep(1)
    
    async def process_message(self, message: CommunicationMessage):
        """Process incoming message"""
        try:
            # Verify message signature
            if not self.verify_message_signature(message):
                logging.warning(f"Invalid message signature from {message.sender_id}")
                return
            
            # Check message TTL
            if self.is_message_expired(message):
                logging.warning(f"Message expired from {message.sender_id}")
                return
            
            # Decrypt message if needed
            if message.encryption_key:
                message = self.decrypt_message(message)
            
            # Route message to appropriate handler
            handler = self.message_handlers.get(message.message_type)
            if handler:
                await handler(message)
            else:
                logging.warning(f"No handler for message type: {message.message_type}")
            
        except Exception as e:
            logging.error(f"Error processing message: {e}")
    
    async def consume_kafka_messages(self):
        """Consume messages from Kafka"""
        try:
            for message in self.kafka_consumer:
                try:
                    message_data = message.value
                    comm_message = CommunicationMessage(**message_data)
                    await self.message_queue.put(comm_message)
                    
                except Exception as e:
                    logging.error(f"Error processing Kafka message: {e}")
                    
        except Exception as e:
            logging.error(f"Error consuming Kafka messages: {e}")
    
    async def start_websocket_server(self):
        """Start WebSocket server"""
        try:
            async def handle_websocket(websocket, path):
                client_id = f"client_{uuid.uuid4()}"
                self.websocket_clients[client_id] = websocket
                
                try:
                    async for message in websocket:
                        # Handle WebSocket message
                        await self.handle_websocket_message(client_id, message)
                        
                except websockets.exceptions.ConnectionClosed:
                    del self.websocket_clients[client_id]
            
            self.websocket_server = await websockets.serve(
                handle_websocket, "localhost", self.websocket_port
            )
            
        except Exception as e:
            logging.error(f"Failed to start WebSocket server: {e}")
    
    async def handle_websocket_message(self, client_id: str, message: str):
        """Handle WebSocket message"""
        try:
            message_data = json.loads(message)
            comm_message = CommunicationMessage(**message_data)
            await self.message_queue.put(comm_message)
            
        except Exception as e:
            logging.error(f"Error handling WebSocket message: {e}")
    
    async def discover_peers(self):
        """Discover other agents in the network"""
        try:
            # Broadcast discovery message
            discovery_message = {
                'agent_id': self.agent_id,
                'capabilities': self.capabilities,
                'endpoints': {
                    'zmq': f"tcp://localhost:{5550 + hash(self.agent_id) % 1000}",
                    'websocket': f"ws://localhost:{self.websocket_port}",
                    'kafka': f"agent_{self.agent_id}"
                }
            }
            
            await self.send_message(
                "broadcast",
                "peer_discovery",
                discovery_message,
                ProtocolType.BROADCAST
            )
            
        except Exception as e:
            logging.error(f"Error discovering peers: {e}")
    
    def register_message_handler(self, message_type: str, handler: Callable):
        """Register message handler"""
        self.message_handlers[message_type] = handler
    
    def register_capability(self, capability: AgentCapability):
        """Register agent capability"""
        self.capabilities[capability.capability_type] = capability
    
    def sign_message(self, message: CommunicationMessage) -> str:
        """Sign message with private key"""
        try:
            message_data = f"{message.message_id}{message.sender_id}{message.receiver_id}{json.dumps(message.content, sort_keys=True)}"
            
            from cryptography.hazmat.primitives import hashes
            from cryptography.hazmat.primitives.asymmetric import padding
            
            signature = self.private_key.sign(
                message_data.encode(),
                padding.PSS(
                    mgf=padding.MGF1(hashes.SHA256()),
                    salt_length=padding.PSS.MAX_LENGTH
                ),
                hashes.SHA256()
            )
            
            return base64.b64encode(signature).decode()
            
        except Exception as e:
            logging.error(f"Failed to sign message: {e}")
            return ""
    
    def verify_message_signature(self, message: CommunicationMessage) -> bool:
        """Verify message signature"""
        try:
            if not message.signature:
                return False
            
            message_data = f"{message.message_id}{message.sender_id}{message.receiver_id}{json.dumps(message.content, sort_keys=True)}"
            signature = base64.b64decode(message.signature)
            
            # Get sender's public key (would be from peer registry)
            sender_public_key = self.get_peer_public_key(message.sender_id)
            if not sender_public_key:
                return False
            
            from cryptography.hazmat.primitives import hashes
            from cryptography.hazmat.primitives.asymmetric import padding
            
            sender_public_key.verify(
                signature,
                message_data.encode(),
                padding.PSS(
                    mgf=padding.MGF1(hashes.SHA256()),
                    salt_length=padding.PSS.MAX_LENGTH
                ),
                hashes.SHA256()
            )
            
            return True
            
        except Exception as e:
            logging.error(f"Failed to verify message signature: {e}")
            return False
    
    def encrypt_message(self, message: CommunicationMessage, receiver_id: str) -> CommunicationMessage:
        """Encrypt message for specific receiver"""
        try:
            # Get shared secret
            shared_secret = self.shared_secrets.get(receiver_id)
            if not shared_secret:
                return message
            
            # Encrypt content
            from cryptography.fernet import Fernet
            key = base64.urlsafe_b64encode(shared_secret.encode()[:32].ljust(32, b'0'))
            fernet = Fernet(key)
            
            encrypted_content = fernet.encrypt(json.dumps(message.content).encode())
            message.content = {'encrypted_data': base64.b64encode(encrypted_content).decode()}
            message.encryption_key = receiver_id
            
            return message
            
        except Exception as e:
            logging.error(f"Failed to encrypt message: {e}")
            return message
    
    def decrypt_message(self, message: CommunicationMessage) -> CommunicationMessage:
        """Decrypt message"""
        try:
            if not message.encryption_key:
                return message
            
            # Get shared secret
            shared_secret = self.shared_secrets.get(message.encryption_key)
            if not shared_secret:
                return message
            
            # Decrypt content
            from cryptography.fernet import Fernet
            key = base64.urlsafe_b64encode(shared_secret.encode()[:32].ljust(32, b'0'))
            fernet = Fernet(key)
            
            encrypted_data = base64.b64decode(message.content['encrypted_data'])
            decrypted_content = fernet.decrypt(encrypted_data)
            message.content = json.loads(decrypted_content.decode())
            message.encryption_key = None
            
            return message
            
        except Exception as e:
            logging.error(f"Failed to decrypt message: {e}")
            return message
    
    def is_message_expired(self, message: CommunicationMessage) -> bool:
        """Check if message has expired"""
        try:
            message_age = (datetime.now() - message.timestamp).total_seconds()
            return message_age > message.ttl
            
        except Exception as e:
            logging.error(f"Failed to check message expiration: {e}")
            return True
    
    def get_peer_public_key(self, peer_id: str):
        """Get peer's public key"""
        try:
            # This would retrieve the public key from peer registry
            # For now, returning None
            return None
            
        except Exception as e:
            logging.error(f"Failed to get peer public key: {e}")
            return None
    
    async def handle_response(self, response: CommunicationMessage):
        """Handle response message"""
        try:
            # Check if we're waiting for this response
            if response.correlation_id in self.response_waiters:
                waiter = self.response_waiters.pop(response.correlation_id)
                waiter.set_result(response)
            
        except Exception as e:
            logging.error(f"Error handling response: {e}")
    
    async def wait_for_response(self, correlation_id: str, timeout: int = 30) -> Optional[CommunicationMessage]:
        """Wait for response with correlation ID"""
        try:
            # Create future for response
            future = asyncio.Future()
            self.response_waiters[correlation_id] = future
            
            # Wait for response with timeout
            try:
                response = await asyncio.wait_for(future, timeout=timeout)
                return response
            except asyncio.TimeoutError:
                self.response_waiters.pop(correlation_id, None)
                return None
            
        except Exception as e:
            logging.error(f"Error waiting for response: {e}")
            return None

# Example usage
async def main():
    """Example usage of Agent Communication Protocol"""
    config = {
        'kafka': {
            'bootstrap_servers': ['localhost:9092']
        },
        'redis': {
            'host': 'localhost',
            'port': 6379,
            'db': 0
        }
    }
    
    protocol = AgentCommunicationProtocol("test_agent_001", config)
    
    # Register message handlers
    async def handle_data_request(message: CommunicationMessage):
        print(f"Received data request: {message.content}")
    
    async def handle_analysis_request(message: CommunicationMessage):
        print(f"Received analysis request: {message.content}")
    
    protocol.register_message_handler("data_request", handle_data_request)
    protocol.register_message_handler("analysis_request", handle_analysis_request)
    
    try:
        await protocol.start_protocol()
        
        # Send test message
        await protocol.send_message(
            "other_agent_001",
            "data_request",
            {"request_type": "user_data", "user_id": "123"},
            ProtocolType.REQUEST_RESPONSE
        )
        
        # Keep running
        while True:
            await asyncio.sleep(1)
            
    except KeyboardInterrupt:
        await protocol.stop_protocol()

if __name__ == "__main__":
    asyncio.run(main())
