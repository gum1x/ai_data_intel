"""
Real-Time Streaming Engine - Apache Kafka and Spark for high-throughput data processing
"""

import asyncio
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Callable
import logging
from dataclasses import dataclass, asdict
from enum import Enum
import uuid

from kafka import KafkaProducer, KafkaConsumer
from kafka.errors import KafkaError

from pyspark.sql import SparkSession
from pyspark.sql.functions import *
from pyspark.sql.types import *
from pyspark.streaming import StreamingContext
from pyspark.streaming.kafka import KafkaUtils

import redis
import pickle

import zmq
import threading

class StreamType(Enum):
    TELEGRAM_MESSAGES = "telegram_messages"
    USER_ACTIVITY = "user_activity"
    THREAT_ALERTS = "threat_alerts"
    BEHAVIOR_ANALYTICS = "behavior_analytics"
    NETWORK_EVENTS = "network_events"
    AI_INSIGHTS = "ai_insights"

@dataclass
class StreamMessage:
    """Stream message structure"""
    message_id: str
    stream_type: StreamType
    timestamp: datetime
    data: Dict[str, Any]
    source: str
    priority: int = 1
    metadata: Dict[str, Any] = None

class RealTimeStreamingEngine:
    """Real-time streaming engine with Kafka and Spark"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.kafka_producer = None
        self.kafka_consumers = {}
        self.spark_session = None
        self.streaming_context = None
        self.redis_client = None
        self.zmq_context = None
        self.zmq_sockets = {}
        self.message_handlers = {}
        self.streaming_tasks = []
        self.is_running = False
        
        self.initialize_components()
    
    def initialize_components(self):
        """Initialize all streaming components"""
        try:
            self.kafka_producer = KafkaProducer(
                bootstrap_servers=self.config['kafka']['bootstrap_servers'],
                value_serializer=lambda v: json.dumps(v).encode('utf-8'),
                key_serializer=lambda k: k.encode('utf-8') if k else None,
                acks='all',
                retries=3,
                batch_size=16384,
                linger_ms=10,
                compression_type='gzip'
            )
            
            self.redis_client = redis.Redis(
                host=self.config['redis']['host'],
                port=self.config['redis']['port'],
                db=self.config['redis']['db'],
                decode_responses=True
            )
            
            self.zmq_context = zmq.Context()
            
            self.initialize_spark()
            
            logging.info("Real-time streaming engine initialized successfully")
            
        except Exception as e:
            logging.error(f"Failed to initialize streaming engine: {e}")
            raise
    
    def initialize_spark(self):
        """Initialize Spark session and streaming context"""
        try:
            self.spark_session = SparkSession.builder \
                .appName("AI_Data_Intelligence_Streaming") \
                .config("spark.sql.adaptive.enabled", "true") \
                .config("spark.sql.adaptive.coalescePartitions.enabled", "true") \
                .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer") \
                .config("spark.sql.streaming.checkpointLocation", "/tmp/checkpoint") \
                .getOrCreate()
            
            self.spark_session.sparkContext.setLogLevel("WARN")
            
            logging.info("Spark session initialized successfully")
            
        except Exception as e:
            logging.error(f"Failed to initialize Spark: {e}")
            raise
    
    async def start_streaming(self):
        """Start all streaming processes"""
        try:
            self.is_running = True
            
            await self.start_kafka_consumers()
            
            await self.start_spark_streaming()
            
            await self.start_zmq_handlers()
            
            await self.start_real_time_analytics()
            
            logging.info("All streaming processes started successfully")
            
        except Exception as e:
            logging.error(f"Failed to start streaming: {e}")
            raise
    
    async def start_kafka_consumers(self):
        """Start Kafka consumers for different stream types"""
        for stream_type in StreamType:
            consumer = KafkaConsumer(
                stream_type.value,
                bootstrap_servers=self.config['kafka']['bootstrap_servers'],
                value_deserializer=lambda m: json.loads(m.decode('utf-8')),
                group_id=f"ai_intelligence_{stream_type.value}",
                auto_offset_reset='latest',
                enable_auto_commit=True,
                max_poll_records=1000
            )
            
            self.kafka_consumers[stream_type] = consumer
            
            task = asyncio.create_task(self.consume_messages(stream_type, consumer))
            self.streaming_tasks.append(task)
    
    async def start_spark_streaming(self):
        """Start Spark streaming for real-time analytics"""
        try:
            ssc = StreamingContext(self.spark_session.sparkContext, batchDuration=5)
            
            kafka_stream = KafkaUtils.createDirectStream(
                ssc,
                [stream_type.value for stream_type in StreamType],
                {
                    "metadata.broker.list": self.config['kafka']['bootstrap_servers'],
                    "auto.offset.reset": "latest"
                }
            )
            
            kafka_stream.foreachRDD(self.process_spark_rdd)
            
            ssc.start()
            self.streaming_context = ssc
            
            logging.info("Spark streaming started successfully")
            
        except Exception as e:
            logging.error(f"Failed to start Spark streaming: {e}")
            raise
    
    async def start_zmq_handlers(self):
        """Start ZMQ message handlers"""
        try:
            for stream_type in StreamType:
                socket = self.zmq_context.socket(zmq.PULL)
                socket.bind(f"tcp://*:{5555 + list(StreamType).index(stream_type)}")
                self.zmq_sockets[stream_type] = socket
                
                task = asyncio.create_task(self.handle_zmq_messages(stream_type, socket))
                self.streaming_tasks.append(task)
            
            logging.info("ZMQ handlers started successfully")
            
        except Exception as e:
            logging.error(f"Failed to start ZMQ handlers: {e}")
            raise
    
    async def start_real_time_analytics(self):
        """Start real-time analytics processing"""
        try:
            analytics_tasks = [
                asyncio.create_task(self.real_time_behavior_analysis()),
                asyncio.create_task(self.real_time_threat_detection()),
                asyncio.create_task(self.real_time_user_profiling()),
                asyncio.create_task(self.real_time_network_analysis()),
                asyncio.create_task(self.real_time_anomaly_detection())
            ]
            
            self.streaming_tasks.extend(analytics_tasks)
            
            logging.info("Real-time analytics started successfully")
            
        except Exception as e:
            logging.error(f"Failed to start real-time analytics: {e}")
            raise
    
    async def consume_messages(self, stream_type: StreamType, consumer: KafkaConsumer):
        """Consume messages from Kafka"""
        try:
            while self.is_running:
                message_batch = consumer.poll(timeout_ms=1000)
                
                for topic_partition, messages in message_batch.items():
                    for message in messages:
                        try:
                            await self.process_message(stream_type, message.value)
                            
                        except Exception as e:
                            logging.error(f"Error processing message: {e}")
                
                await asyncio.sleep(0.1)
                
        except Exception as e:
            logging.error(f"Error consuming messages: {e}")
    
    async def process_message(self, stream_type: StreamType, message_data: Dict[str, Any]):
        """Process individual message"""
        try:
            stream_message = StreamMessage(
                message_id=str(uuid.uuid4()),
                stream_type=stream_type,
                timestamp=datetime.now(),
                data=message_data,
                source="kafka"
            )
            
            await self.cache_message(stream_message)
            
            if stream_type == StreamType.TELEGRAM_MESSAGES:
                await self.process_telegram_message(stream_message)
            elif stream_type == StreamType.USER_ACTIVITY:
                await self.process_user_activity(stream_message)
            elif stream_type == StreamType.THREAT_ALERTS:
                await self.process_threat_alert(stream_message)
            elif stream_type == StreamType.BEHAVIOR_ANALYTICS:
                await self.process_behavior_analytics(stream_message)
            elif stream_type == StreamType.NETWORK_EVENTS:
                await self.process_network_event(stream_message)
            elif stream_type == StreamType.AI_INSIGHTS:
                await self.process_ai_insights(stream_message)
            
        except Exception as e:
            logging.error(f"Error processing message: {e}")
    
    async def process_telegram_message(self, message: StreamMessage):
        """Process Telegram message"""
        try:
            message_data = message.data
            
            analysis_results = {
                'sentiment': await self.analyze_sentiment(message_data.get('text', '')),
                'threat_level': await self.assess_threat_level(message_data),
                'behavioral_indicators': await self.extract_behavioral_indicators(message_data),
                'network_connections': await self.analyze_network_connections(message_data)
            }
            
            await self.publish_message(StreamType.AI_INSIGHTS, analysis_results)
            
        except Exception as e:
            logging.error(f"Error processing Telegram message: {e}")
    
    async def process_user_activity(self, message: StreamMessage):
        """Process user activity"""
        try:
            activity_data = message.data
            
            profile_update = {
                'user_id': activity_data.get('user_id'),
                'activity_type': activity_data.get('activity_type'),
                'timestamp': message.timestamp.isoformat(),
                'behavioral_score': await self.calculate_behavioral_score(activity_data),
                'risk_assessment': await self.assess_user_risk(activity_data)
            }
            
            await self.update_user_profile(profile_update)
            
        except Exception as e:
            logging.error(f"Error processing user activity: {e}")
    
    async def process_threat_alert(self, message: StreamMessage):
        """Process threat alert"""
        try:
            threat_data = message.data
            
            threat_analysis = {
                'threat_id': str(uuid.uuid4()),
                'threat_type': threat_data.get('threat_type'),
                'severity': threat_data.get('severity'),
                'confidence': threat_data.get('confidence'),
                'affected_users': threat_data.get('affected_users', []),
                'recommended_actions': await self.generate_threat_response(threat_data),
                'timestamp': message.timestamp.isoformat()
            }
            
            await self.store_threat_alert(threat_analysis)
            
            await self.notify_security_team(threat_analysis)
            
        except Exception as e:
            logging.error(f"Error processing threat alert: {e}")
    
    async def process_behavior_analytics(self, message: StreamMessage):
        """Process behavior analytics"""
        try:
            behavior_data = message.data
            
            behavior_analysis = {
                'user_id': behavior_data.get('user_id'),
                'behavior_pattern': behavior_data.get('behavior_pattern'),
                'anomaly_score': await self.calculate_anomaly_score(behavior_data),
                'predicted_behavior': await self.predict_behavior(behavior_data),
                'recommendations': await self.generate_behavior_recommendations(behavior_data)
            }
            
            await self.update_behavior_model(behavior_analysis)
            
        except Exception as e:
            logging.error(f"Error processing behavior analytics: {e}")
    
    async def process_network_event(self, message: StreamMessage):
        """Process network event"""
        try:
            network_data = message.data
            
            network_analysis = {
                'event_id': str(uuid.uuid4()),
                'event_type': network_data.get('event_type'),
                'source': network_data.get('source'),
                'target': network_data.get('target'),
                'network_impact': await self.assess_network_impact(network_data),
                'security_implications': await self.assess_security_implications(network_data)
            }
            
            await self.update_network_graph(network_analysis)
            
        except Exception as e:
            logging.error(f"Error processing network event: {e}")
    
    async def process_ai_insights(self, message: StreamMessage):
        """Process AI insights"""
        try:
            insights_data = message.data
            
            await self.store_ai_insights(insights_data)
            
            recommendations = await self.generate_insights_recommendations(insights_data)
            
            await self.publish_message(StreamType.AI_INSIGHTS, recommendations)
            
        except Exception as e:
            logging.error(f"Error processing AI insights: {e}")
    
    async def publish_message(self, stream_type: StreamType, data: Dict[str, Any]):
        """Publish message to Kafka"""
        try:
            message = {
                'message_id': str(uuid.uuid4()),
                'timestamp': datetime.now().isoformat(),
                'data': data
            }
            
            self.kafka_producer.send(
                stream_type.value,
                key=message['message_id'],
                value=message
            )
            
        except Exception as e:
            logging.error(f"Error publishing message: {e}")
    
    async def cache_message(self, message: StreamMessage):
        """Cache message in Redis"""
        try:
            cache_key = f"stream:{message.stream_type.value}:{message.message_id}"
            cache_data = asdict(message)
            cache_data['timestamp'] = cache_data['timestamp'].isoformat()
            
            self.redis_client.setex(
                cache_key,
                3600,
                json.dumps(cache_data)
            )
            
        except Exception as e:
            logging.error(f"Error caching message: {e}")
    
    def process_spark_rdd(self, rdd):
        """Process RDD in Spark streaming"""
        try:
            if not rdd.isEmpty():
                df = self.spark_session.createDataFrame(rdd.map(lambda x: x[1]))
                
                self.perform_spark_analytics(df)
                
        except Exception as e:
            logging.error(f"Error processing Spark RDD: {e}")
    
    def perform_spark_analytics(self, df):
        """Perform real-time analytics with Spark"""
        try:
            aggregations = df.groupBy("stream_type") \
                .agg(
                    count("*").alias("message_count"),
                    avg("data.message_frequency").alias("avg_message_frequency"),
                    max("timestamp").alias("latest_timestamp")
                )
            
            aggregations.write \
                .format("console") \
                .option("truncate", False) \
                .save()
            
        except Exception as e:
            logging.error(f"Error performing Spark analytics: {e}")
    
    async def real_time_behavior_analysis(self):
        """Real-time behavior analysis"""
        while self.is_running:
            try:
                recent_messages = await self.get_recent_messages(StreamType.TELEGRAM_MESSAGES, 100)
                
                behavior_patterns = await self.analyze_behavior_patterns(recent_messages)
                
                await self.publish_message(StreamType.BEHAVIOR_ANALYTICS, behavior_patterns)
                
                await asyncio.sleep(10)
                
            except Exception as e:
                logging.error(f"Error in real-time behavior analysis: {e}")
                await asyncio.sleep(5)
    
    async def real_time_threat_detection(self):
        """Real-time threat detection"""
        while self.is_running:
            try:
                recent_messages = await self.get_recent_messages(StreamType.TELEGRAM_MESSAGES, 50)
                
                threats = await self.detect_threats(recent_messages)
                
                for threat in threats:
                    await self.publish_message(StreamType.THREAT_ALERTS, threat)
                
                await asyncio.sleep(5)
                
            except Exception as e:
                logging.error(f"Error in real-time threat detection: {e}")
                await asyncio.sleep(5)
    
    async def real_time_user_profiling(self):
        """Real-time user profiling"""
        while self.is_running:
            try:
                recent_activities = await self.get_recent_messages(StreamType.USER_ACTIVITY, 200)
                
                for activity in recent_activities:
                    await self.update_user_profile_realtime(activity)
                
                await asyncio.sleep(15)
                
            except Exception as e:
                logging.error(f"Error in real-time user profiling: {e}")
                await asyncio.sleep(5)
    
    async def real_time_network_analysis(self):
        """Real-time network analysis"""
        while self.is_running:
            try:
                recent_events = await self.get_recent_messages(StreamType.NETWORK_EVENTS, 100)
                
                network_analysis = await self.analyze_network_realtime(recent_events)
                
                await self.publish_message(StreamType.AI_INSIGHTS, network_analysis)
                
                await asyncio.sleep(20)
                
            except Exception as e:
                logging.error(f"Error in real-time network analysis: {e}")
                await asyncio.sleep(5)
    
    async def real_time_anomaly_detection(self):
        """Real-time anomaly detection"""
        while self.is_running:
            try:
                recent_data = await self.get_recent_messages(StreamType.BEHAVIOR_ANALYTICS, 150)
                
                anomalies = await self.detect_anomalies(recent_data)
                
                for anomaly in anomalies:
                    await self.publish_message(StreamType.THREAT_ALERTS, anomaly)
                
                await asyncio.sleep(30)
                
            except Exception as e:
                logging.error(f"Error in real-time anomaly detection: {e}")
                await asyncio.sleep(5)
    
    async def get_recent_messages(self, stream_type: StreamType, limit: int) -> List[Dict[str, Any]]:
        """Get recent messages from cache"""
        try:
            pattern = f"stream:{stream_type.value}:*"
            keys = self.redis_client.keys(pattern)
            
            messages = []
            for key in keys[:limit]:
                message_data = self.redis_client.get(key)
                if message_data:
                    messages.append(json.loads(message_data))
            
            return messages
            
        except Exception as e:
            logging.error(f"Error getting recent messages: {e}")
            return []
    
    async def stop_streaming(self):
        """Stop all streaming processes"""
        try:
            self.is_running = False
            
            for task in self.streaming_tasks:
                task.cancel()
            
            for consumer in self.kafka_consumers.values():
                consumer.close()
            
            if self.streaming_context:
                self.streaming_context.stop()
            
            for socket in self.zmq_sockets.values():
                socket.close()
            
            if self.kafka_producer:
                self.kafka_producer.close()
            
            logging.info("All streaming processes stopped successfully")
            
        except Exception as e:
            logging.error(f"Error stopping streaming: {e}")
    
    async def analyze_sentiment(self, text: str) -> str:
        """Analyze sentiment of text"""
        return "neutral"
    
    async def assess_threat_level(self, message_data: Dict[str, Any]) -> str:
        """Assess threat level of message"""
        return "low"
    
    async def extract_behavioral_indicators(self, message_data: Dict[str, Any]) -> List[str]:
        """Extract behavioral indicators from message data"""
        indicators = []
        
        try:
            content = message_data.get('content', '')
            timestamp = message_data.get('timestamp', datetime.now())
            
            # Message frequency analysis
            user_id = message_data.get('user_id', '')
            if user_id in self.user_activity_cache:
                recent_messages = [msg for msg in self.user_activity_cache[user_id] 
                                 if (timestamp - msg['timestamp']).seconds < 3600]
                if len(recent_messages) > 50:  # More than 50 messages per hour
                    indicators.append('high_message_frequency')
            
            # Content analysis
            if len(content) > 1000:
                indicators.append('long_messages')
            
            if content.count('!') > 5:
                indicators.append('excessive_exclamation')
            
            if content.count('?') > 5:
                indicators.append('excessive_questions')
            
            # Time pattern analysis
            hour = timestamp.hour
            if hour < 6 or hour > 23:
                indicators.append('unusual_timing')
            
            # Language analysis
            if any(word in content.lower() for word in ['urgent', 'asap', 'immediately']):
                indicators.append('urgency_language')
            
            # Link analysis
            if content.count('http') > 3:
                indicators.append('excessive_links')
            
            # Emoji analysis
            emoji_count = sum(1 for char in content if ord(char) > 127)
            if emoji_count > 10:
                indicators.append('excessive_emoji')
            
        except Exception as e:
            logging.error(f"Error extracting behavioral indicators: {e}")
        
        return indicators
    
    async def analyze_network_connections(self, message_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze network connections and relationships"""
        try:
            user_id = message_data.get('user_id', '')
            chat_id = message_data.get('chat_id', '')
            
            # Initialize user network if not exists
            if user_id not in self.network_graph:
                self.network_graph[user_id] = {
                    'connections': set(),
                    'chats': set(),
                    'interaction_count': 0,
                    'last_seen': datetime.now()
                }
            
            # Update network data
            network_data = self.network_graph[user_id]
            network_data['chats'].add(chat_id)
            network_data['interaction_count'] += 1
            network_data['last_seen'] = datetime.now()
            
            # Analyze mentions and replies
            content = message_data.get('content', '')
            mentions = self.extract_mentions(content)
            for mention in mentions:
                network_data['connections'].add(mention)
            
            # Calculate network metrics
            connection_count = len(network_data['connections'])
            chat_count = len(network_data['chats'])
            interaction_density = network_data['interaction_count'] / max(connection_count, 1)
            
            return {
                'user_id': user_id,
                'connection_count': connection_count,
                'chat_count': chat_count,
                'interaction_density': interaction_density,
                'connections': list(network_data['connections']),
                'chats': list(network_data['chats']),
                'last_seen': network_data['last_seen'].isoformat()
            }
            
        except Exception as e:
            logging.error(f"Error analyzing network connections: {e}")
            return {'error': str(e)}
    
    async def calculate_behavioral_score(self, activity_data: Dict[str, Any]) -> float:
        """Calculate behavioral score based on activity patterns"""
        try:
            score = 0.5  # Base score
            
            # Message frequency factor
            messages_per_hour = activity_data.get('messages_per_hour', 0)
            if messages_per_hour > 20:
                score += 0.1
            elif messages_per_hour < 1:
                score -= 0.1
            
            # Time consistency factor
            time_consistency = activity_data.get('time_consistency', 0.5)
            score += (time_consistency - 0.5) * 0.2
            
            # Content diversity factor
            content_diversity = activity_data.get('content_diversity', 0.5)
            score += (content_diversity - 0.5) * 0.2
            
            # Network size factor
            network_size = activity_data.get('network_size', 0)
            if network_size > 100:
                score += 0.1
            elif network_size < 5:
                score -= 0.1
            
            # Engagement factor
            engagement_rate = activity_data.get('engagement_rate', 0.5)
            score += (engagement_rate - 0.5) * 0.3
            
            # Normalize score between 0 and 1
            return max(0.0, min(1.0, score))
            
        except Exception as e:
            logging.error(f"Error calculating behavioral score: {e}")
            return 0.5
    
    async def assess_user_risk(self, activity_data: Dict[str, Any]) -> str:
        """Assess user risk level based on activity"""
        try:
            behavioral_score = activity_data.get('behavioral_score', 0.5)
            suspicious_indicators = activity_data.get('suspicious_indicators', [])
            
            risk_score = behavioral_score
            
            # Adjust based on suspicious indicators
            for indicator in suspicious_indicators:
                if indicator in ['high_message_frequency', 'excessive_links', 'urgency_language']:
                    risk_score += 0.1
                elif indicator in ['unusual_timing', 'excessive_emoji']:
                    risk_score += 0.05
            
            # Determine risk level
            if risk_score > 0.8:
                return "high"
            elif risk_score > 0.6:
                return "medium"
            else:
                return "low"
                
        except Exception as e:
            logging.error(f"Error assessing user risk: {e}")
            return "unknown"
    
    async def generate_threat_response(self, threat_data: Dict[str, Any]) -> List[str]:
        """Generate threat response recommendations"""
        try:
            threat_level = threat_data.get('threat_level', 'low')
            threat_type = threat_data.get('threat_type', 'unknown')
            user_id = threat_data.get('user_id', '')
            
            recommendations = []
            
            if threat_level == "high":
                recommendations.extend([
                    "Immediate user account suspension",
                    "Notify security team",
                    "Preserve all evidence",
                    "Block all network connections",
                    "Initiate forensic analysis"
                ])
            elif threat_level == "medium":
                recommendations.extend([
                    "Enhanced monitoring",
                    "Restrict user permissions",
                    "Document suspicious activity",
                    "Monitor network connections"
                ])
            else:
                recommendations.extend([
                    "Continue normal monitoring",
                    "Log activity for future reference"
                ])
            
            # Add specific recommendations based on threat type
            if threat_type == "spam":
                recommendations.append("Implement anti-spam filters")
            elif threat_type == "phishing":
                recommendations.append("Block malicious links")
            elif threat_type == "bot":
                recommendations.append("Verify user identity")
            
            return recommendations
            
        except Exception as e:
            logging.error(f"Error generating threat response: {e}")
            return ["Investigate further"]
    
    async def store_threat_alert(self, threat_analysis: Dict[str, Any]):
        """Store threat alert in database and cache"""
        try:
            alert_id = str(uuid.uuid4())
            threat_analysis['alert_id'] = alert_id
            threat_analysis['created_at'] = datetime.now().isoformat()
            
            # Store in Redis cache
            await self.redis_client.setex(
                f"threat_alert:{alert_id}",
                3600,  # 1 hour TTL
                json.dumps(threat_analysis)
            )
            
            # Store in database
            if hasattr(self, 'database'):
                await self.database.store_threat_alert(threat_analysis)
            
            logging.info(f"Stored threat alert: {alert_id}")
            
        except Exception as e:
            logging.error(f"Error storing threat alert: {e}")
    
    async def notify_security_team(self, threat_analysis: Dict[str, Any]):
        """Notify security team of threat"""
        try:
            threat_level = threat_analysis.get('threat_level', 'low')
            
            # Only notify for medium and high threats
            if threat_level in ['medium', 'high']:
                notification = {
                    'type': 'threat_alert',
                    'threat_level': threat_level,
                    'user_id': threat_analysis.get('user_id', ''),
                    'timestamp': datetime.now().isoformat(),
                    'recommendations': threat_analysis.get('recommendations', [])
                }
                
                # Send to notification queue
                await self.produce_message(
                    StreamType.THREAT_ALERTS,
                    notification
                )
                
                logging.info(f"Notified security team of {threat_level} threat")
                
        except Exception as e:
            logging.error(f"Error notifying security team: {e}")
    
    async def calculate_anomaly_score(self, behavior_data: Dict[str, Any]) -> float:
        """Calculate anomaly score for behavior data"""
        try:
            score = 0.0
            
            # Message frequency anomaly
            current_frequency = behavior_data.get('message_frequency', 0)
            baseline_frequency = behavior_data.get('baseline_frequency', 1)
            
            if current_frequency > baseline_frequency * 3:
                score += 0.3
            elif current_frequency < baseline_frequency * 0.1:
                score += 0.2
            
            # Time pattern anomaly
            unusual_timing = behavior_data.get('unusual_timing', False)
            if unusual_timing:
                score += 0.2
            
            # Content anomaly
            content_anomaly = behavior_data.get('content_anomaly', False)
            if content_anomaly:
                score += 0.2
            
            # Network anomaly
            network_anomaly = behavior_data.get('network_anomaly', False)
            if network_anomaly:
                score += 0.3
            
            return min(1.0, score)
            
        except Exception as e:
            logging.error(f"Error calculating anomaly score: {e}")
            return 0.0
    
    async def predict_behavior(self, behavior_data: Dict[str, Any]) -> str:
        """Predict future behavior based on current patterns"""
        try:
            behavioral_score = behavior_data.get('behavioral_score', 0.5)
            anomaly_score = behavior_data.get('anomaly_score', 0.0)
            risk_level = behavior_data.get('risk_level', 'low')
            
            # Simple prediction logic
            if anomaly_score > 0.7:
                return "escalating_risk"
            elif behavioral_score > 0.8:
                return "high_activity"
            elif behavioral_score < 0.3:
                return "low_activity"
            elif risk_level == "high":
                return "threat_behavior"
            else:
                return "normal"
                
        except Exception as e:
            logging.error(f"Error predicting behavior: {e}")
            return "unknown"
    
    async def generate_behavior_recommendations(self, behavior_data: Dict[str, Any]) -> List[str]:
        """Generate behavior-based recommendations"""
        try:
            predictions = behavior_data.get('predictions', [])
            risk_level = behavior_data.get('risk_level', 'low')
            
            recommendations = []
            
            for prediction in predictions:
                if prediction == "escalating_risk":
                    recommendations.extend([
                        "Increase monitoring frequency",
                        "Prepare incident response",
                        "Document all activities"
                    ])
                elif prediction == "high_activity":
                    recommendations.extend([
                        "Monitor for fatigue",
                        "Check for automation",
                        "Verify user identity"
                    ])
                elif prediction == "threat_behavior":
                    recommendations.extend([
                        "Immediate investigation",
                        "Restrict access",
                        "Notify security team"
                    ])
            
            if risk_level == "high":
                recommendations.append("Consider account suspension")
            
            return list(set(recommendations))  # Remove duplicates
            
        except Exception as e:
            logging.error(f"Error generating behavior recommendations: {e}")
            return ["Continue monitoring"]
    
    async def update_behavior_model(self, behavior_analysis: Dict[str, Any]):
        """Update behavior model with new data"""
        try:
            user_id = behavior_analysis.get('user_id', '')
            if not user_id:
                return
            
            # Update user behavior cache
            if user_id not in self.behavior_models:
                self.behavior_models[user_id] = {
                    'baseline_frequency': 0,
                    'typical_timing': [],
                    'content_patterns': [],
                    'network_size': 0,
                    'last_updated': datetime.now()
                }
            
            model = self.behavior_models[user_id]
            
            # Update baseline frequency
            current_frequency = behavior_analysis.get('message_frequency', 0)
            if model['baseline_frequency'] == 0:
                model['baseline_frequency'] = current_frequency
            else:
                # Exponential moving average
                model['baseline_frequency'] = 0.9 * model['baseline_frequency'] + 0.1 * current_frequency
            
            # Update typical timing
            timestamp = behavior_analysis.get('timestamp', datetime.now())
            if isinstance(timestamp, str):
                timestamp = datetime.fromisoformat(timestamp)
            model['typical_timing'].append(timestamp.hour)
            
            # Keep only last 100 timing records
            if len(model['typical_timing']) > 100:
                model['typical_timing'] = model['typical_timing'][-100:]
            
            # Update content patterns
            content_patterns = behavior_analysis.get('content_patterns', [])
            model['content_patterns'].extend(content_patterns)
            
            # Keep only last 50 patterns
            if len(model['content_patterns']) > 50:
                model['content_patterns'] = model['content_patterns'][-50:]
            
            # Update network size
            network_size = behavior_analysis.get('network_size', 0)
            if network_size > model['network_size']:
                model['network_size'] = network_size
            
            model['last_updated'] = datetime.now()
            
            logging.info(f"Updated behavior model for user: {user_id}")
            
        except Exception as e:
            logging.error(f"Error updating behavior model: {e}")
    
    async def assess_network_impact(self, network_data: Dict[str, Any]) -> str:
        """Assess network impact of user activity"""
        try:
            user_id = network_data.get('user_id', '')
            connection_count = network_data.get('connection_count', 0)
            interaction_density = network_data.get('interaction_density', 0)
            
            # Calculate impact score
            impact_score = 0.0
            
            # Connection count impact
            if connection_count > 500:
                impact_score += 0.4
            elif connection_count > 100:
                impact_score += 0.2
            
            # Interaction density impact
            if interaction_density > 10:
                impact_score += 0.3
            elif interaction_density > 5:
                impact_score += 0.1
            
            # Determine impact level
            if impact_score > 0.6:
                return "high"
            elif impact_score > 0.3:
                return "medium"
            else:
                return "low"
                
        except Exception as e:
            logging.error(f"Error assessing network impact: {e}")
            return "unknown"
    
    async def assess_security_implications(self, network_data: Dict[str, Any]) -> List[str]:
        """Assess security implications of network activity"""
        try:
            implications = []
            
            connection_count = network_data.get('connection_count', 0)
            chat_count = network_data.get('chat_count', 0)
            interaction_density = network_data.get('interaction_density', 0)
            
            # High connection count implications
            if connection_count > 1000:
                implications.extend([
                    "Potential bot network",
                    "High influence capability",
                    "Difficult to monitor all connections"
                ])
            
            # High chat count implications
            if chat_count > 100:
                implications.extend([
                    "Multi-channel presence",
                    "Potential cross-channel coordination",
                    "Increased monitoring complexity"
                ])
            
            # High interaction density implications
            if interaction_density > 20:
                implications.extend([
                    "Potential automation",
                    "High engagement manipulation",
                    "Possible coordinated activity"
                ])
            
            return implications
            
        except Exception as e:
            logging.error(f"Error assessing security implications: {e}")
            return ["Unknown implications"]
    
    async def update_network_graph(self, network_analysis: Dict[str, Any]):
        """Update network graph with new analysis"""
        try:
            user_id = network_analysis.get('user_id', '')
            connections = network_analysis.get('connections', [])
            
            if user_id not in self.network_graph:
                self.network_graph[user_id] = {
                    'connections': set(),
                    'chats': set(),
                    'interaction_count': 0,
                    'last_seen': datetime.now()
                }
            
            # Update connections
            self.network_graph[user_id]['connections'].update(connections)
            
            # Update interaction count
            self.network_graph[user_id]['interaction_count'] += 1
            self.network_graph[user_id]['last_seen'] = datetime.now()
            
            logging.info(f"Updated network graph for user: {user_id}")
            
        except Exception as e:
            logging.error(f"Error updating network graph: {e}")
    
    async def store_ai_insights(self, insights_data: Dict[str, Any]):
        """Store AI insights in database and cache"""
        try:
            insight_id = str(uuid.uuid4())
            insights_data['insight_id'] = insight_id
            insights_data['created_at'] = datetime.now().isoformat()
            
            # Store in Redis cache
            await self.redis_client.setex(
                f"ai_insight:{insight_id}",
                7200,  # 2 hours TTL
                json.dumps(insights_data)
            )
            
            # Store in database
            if hasattr(self, 'database'):
                await self.database.store_ai_insight(insights_data)
            
            logging.info(f"Stored AI insight: {insight_id}")
            
        except Exception as e:
            logging.error(f"Error storing AI insights: {e}")
    
    async def generate_insights_recommendations(self, insights_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate insights-based recommendations"""
        try:
            insights_type = insights_data.get('type', 'general')
            confidence = insights_data.get('confidence', 0.5)
            data = insights_data.get('data', {})
            
            recommendations = {
                'immediate_actions': [],
                'monitoring_suggestions': [],
                'investigation_areas': [],
                'risk_mitigation': []
            }
            
            # Generate recommendations based on insight type
            if insights_type == 'behavior_anomaly':
                if confidence > 0.8:
                    recommendations['immediate_actions'].extend([
                        "Investigate user immediately",
                        "Increase monitoring frequency",
                        "Document all activities"
                    ])
                else:
                    recommendations['monitoring_suggestions'].extend([
                        "Continue enhanced monitoring",
                        "Track behavior patterns"
                    ])
            
            elif insights_type == 'network_threat':
                recommendations['immediate_actions'].extend([
                    "Isolate affected users",
                    "Notify security team",
                    "Preserve evidence"
                ])
                recommendations['risk_mitigation'].extend([
                    "Implement network segmentation",
                    "Enhance access controls"
                ])
            
            elif insights_type == 'content_analysis':
                if data.get('suspicious_content', False):
                    recommendations['investigation_areas'].extend([
                        "Content source verification",
                        "User identity confirmation",
                        "Message pattern analysis"
                    ])
            
            return recommendations
            
        except Exception as e:
            logging.error(f"Error generating insights recommendations: {e}")
            return {'error': str(e)}
    
    async def analyze_behavior_patterns(self, messages: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze behavior patterns from message data"""
        try:
            if not messages:
                return {'error': 'No messages to analyze'}
            
            # Extract patterns
            patterns = {
                'message_frequency': len(messages),
                'time_patterns': [],
                'content_patterns': [],
                'length_patterns': [],
                'language_patterns': []
            }
            
            for message in messages:
                # Time patterns
                timestamp = message.get('timestamp')
                if timestamp:
                    if isinstance(timestamp, str):
                        timestamp = datetime.fromisoformat(timestamp)
                    patterns['time_patterns'].append(timestamp.hour)
                
                # Content patterns
                content = message.get('content', '')
                patterns['content_patterns'].append(len(content))
                
                # Language patterns
                if content:
                    patterns['language_patterns'].extend(content.split())
            
            # Analyze patterns
            analysis = {
                'total_messages': len(messages),
                'average_message_length': sum(patterns['content_patterns']) / len(patterns['content_patterns']) if patterns['content_patterns'] else 0,
                'most_active_hour': max(set(patterns['time_patterns']), key=patterns['time_patterns'].count) if patterns['time_patterns'] else 0,
                'unique_words': len(set(patterns['language_patterns'])),
                'pattern_consistency': self.calculate_pattern_consistency(patterns)
            }
            
            return analysis
            
        except Exception as e:
            logging.error(f"Error analyzing behavior patterns: {e}")
            return {'error': str(e)}
    
    def calculate_pattern_consistency(self, patterns: Dict[str, Any]) -> float:
        """Calculate consistency score for behavior patterns"""
        try:
            if not patterns['time_patterns']:
                return 0.0
            
            # Calculate time pattern consistency
            time_consistency = 0.0
            hour_counts = {}
            for hour in patterns['time_patterns']:
                hour_counts[hour] = hour_counts.get(hour, 0) + 1
            
            if hour_counts:
                max_count = max(hour_counts.values())
                total_count = sum(hour_counts.values())
                time_consistency = max_count / total_count
            
            return time_consistency
            
        except Exception as e:
            logging.error(f"Error calculating pattern consistency: {e}")
            return 0.0
    
    async def detect_threats(self, messages: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Detect threats from message data"""
        try:
            threats = []
            
            for message in messages:
                content = message.get('content', '').lower()
                user_id = message.get('user_id', '')
                
                # Check for threat indicators
                threat_indicators = [
                    'bomb', 'explosive', 'attack', 'kill', 'murder',
                    'terrorist', 'hack', 'breach', 'steal', 'fraud'
                ]
                
                for indicator in threat_indicators:
                    if indicator in content:
                        threats.append({
                            'type': 'content_threat',
                            'severity': 'high',
                            'indicator': indicator,
                            'user_id': user_id,
                            'message_id': message.get('message_id', ''),
                            'timestamp': message.get('timestamp', datetime.now().isoformat())
                        })
                
                # Check for suspicious patterns
                if len(content) > 1000 and content.count('http') > 5:
                    threats.append({
                        'type': 'spam_threat',
                        'severity': 'medium',
                        'indicator': 'excessive_links',
                        'user_id': user_id,
                        'message_id': message.get('message_id', ''),
                        'timestamp': message.get('timestamp', datetime.now().isoformat())
                    })
            
            return threats
            
        except Exception as e:
            logging.error(f"Error detecting threats: {e}")
            return []
    
    async def update_user_profile_realtime(self, activity: Dict[str, Any]):
        """Update user profile in real-time"""
        try:
            user_id = activity.get('user_id', '')
            if not user_id:
                return
            
            # Initialize profile if not exists
            if user_id not in self.user_profiles:
                self.user_profiles[user_id] = {
                    'total_messages': 0,
                    'last_activity': datetime.now(),
                    'behavioral_score': 0.5,
                    'risk_level': 'low',
                    'suspicious_indicators': []
                }
            
            profile = self.user_profiles[user_id]
            
            # Update profile data
            profile['total_messages'] += 1
            profile['last_activity'] = datetime.now()
            
            # Update behavioral score
            behavioral_score = activity.get('behavioral_score', 0.5)
            profile['behavioral_score'] = 0.9 * profile['behavioral_score'] + 0.1 * behavioral_score
            
            # Update risk level
            risk_level = activity.get('risk_level', 'low')
            profile['risk_level'] = risk_level
            
            # Update suspicious indicators
            new_indicators = activity.get('suspicious_indicators', [])
            profile['suspicious_indicators'].extend(new_indicators)
            profile['suspicious_indicators'] = list(set(profile['suspicious_indicators']))
            
            logging.info(f"Updated profile for user: {user_id}")
            
        except Exception as e:
            logging.error(f"Error updating user profile: {e}")
    
    async def analyze_network_realtime(self, events: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze network in real-time"""
        try:
            analysis = {
                'total_events': len(events),
                'unique_users': set(),
                'unique_chats': set(),
                'event_types': {},
                'timeline': []
            }
            
            for event in events:
                # Extract event data
                user_id = event.get('user_id', '')
                chat_id = event.get('chat_id', '')
                event_type = event.get('type', 'unknown')
                timestamp = event.get('timestamp', datetime.now().isoformat())
                
                # Update analysis
                analysis['unique_users'].add(user_id)
                analysis['unique_chats'].add(chat_id)
                analysis['event_types'][event_type] = analysis['event_types'].get(event_type, 0) + 1
                analysis['timeline'].append({
                    'timestamp': timestamp,
                    'user_id': user_id,
                    'event_type': event_type
                })
            
            # Convert sets to lists for JSON serialization
            analysis['unique_users'] = list(analysis['unique_users'])
            analysis['unique_chats'] = list(analysis['unique_chats'])
            
            # Sort timeline by timestamp
            analysis['timeline'].sort(key=lambda x: x['timestamp'])
            
            return analysis
            
        except Exception as e:
            logging.error(f"Error analyzing network realtime: {e}")
            return {'error': str(e)}
    
    async def detect_anomalies(self, data: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Detect anomalies in data"""
        try:
            anomalies = []
            
            if not data:
                return anomalies
            
            # Message frequency anomaly
            message_counts = {}
            for item in data:
                user_id = item.get('user_id', '')
                message_counts[user_id] = message_counts.get(user_id, 0) + 1
            
            # Find users with unusually high message counts
            avg_messages = sum(message_counts.values()) / len(message_counts) if message_counts else 0
            for user_id, count in message_counts.items():
                if count > avg_messages * 3:  # 3x average
                    anomalies.append({
                        'type': 'message_frequency_anomaly',
                        'user_id': user_id,
                        'value': count,
                        'threshold': avg_messages * 3,
                        'severity': 'high'
                    })
            
            # Time pattern anomaly
            timestamps = [item.get('timestamp') for item in data if item.get('timestamp')]
            if len(timestamps) > 10:
                # Check for unusual time clustering
                hour_counts = {}
                for timestamp in timestamps:
                    if isinstance(timestamp, str):
                        timestamp = datetime.fromisoformat(timestamp)
                    hour = timestamp.hour
                    hour_counts[hour] = hour_counts.get(hour, 0) + 1
                
                max_hour_count = max(hour_counts.values()) if hour_counts else 0
                total_count = sum(hour_counts.values())
                
                if max_hour_count > total_count * 0.7:  # 70% of messages in one hour
                    anomalies.append({
                        'type': 'time_clustering_anomaly',
                        'peak_hour': max(hour_counts, key=hour_counts.get),
                        'percentage': (max_hour_count / total_count) * 100,
                        'severity': 'medium'
                    })
            
            return anomalies
            
        except Exception as e:
            logging.error(f"Error detecting anomalies: {e}")
            return []

async def main():
    """Example usage of RealTimeStreamingEngine"""
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
    
    engine = RealTimeStreamingEngine(config)
    
    try:
        await engine.start_streaming()
        
        while True:
            await asyncio.sleep(1)
            
    except KeyboardInterrupt:
        await engine.stop_streaming()

if __name__ == "__main__":
    asyncio.run(main())
