#!/usr/bin/env python3
"""
AI Agents System - Specialized agents for different intelligence tasks with inter-agent communication
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
from abc import ABC, abstractmethod

# Message passing
import zmq
import redis
from kafka import KafkaProducer, KafkaConsumer

class AgentType(Enum):
    INTELLIGENCE_ANALYST = "intelligence_analyst"
    THREAT_HUNTER = "threat_hunter"
    NETWORK_ANALYST = "network_analyst"
    BEHAVIOR_ANALYST = "behavior_analyst"
    DATA_COLLECTOR = "data_collector"
    PATTERN_RECOGNIZER = "pattern_recognizer"
    RISK_ASSESSOR = "risk_assessor"
    COORDINATOR = "coordinator"

class MessageType(Enum):
    DATA_REQUEST = "data_request"
    DATA_RESPONSE = "data_response"
    ANALYSIS_REQUEST = "analysis_request"
    ANALYSIS_RESPONSE = "analysis_response"
    THREAT_ALERT = "threat_alert"
    COORDINATION = "coordination"
    STATUS_UPDATE = "status_update"
    TASK_ASSIGNMENT = "task_assignment"

@dataclass
class AgentMessage:
    """Message structure for inter-agent communication"""
    message_id: str
    sender_id: str
    receiver_id: str
    message_type: MessageType
    content: Dict[str, Any]
    timestamp: datetime
    priority: int = 1
    correlation_id: Optional[str] = None

@dataclass
class AgentCapabilities:
    """Agent capabilities definition"""
    can_analyze: List[str]
    can_collect: List[str]
    can_predict: List[str]
    can_detect: List[str]
    can_coordinate: bool = False

class BaseAgent(ABC):
    """Base class for all AI agents"""
    
    def __init__(self, agent_id: str, agent_type: AgentType, capabilities: AgentCapabilities):
        self.agent_id = agent_id
        self.agent_type = agent_type
        self.capabilities = capabilities
        self.message_queue = queue.Queue()
        self.is_running = False
        self.connections = {}
        self.knowledge_base = {}
        self.task_history = []
        
        # Communication
        self.zmq_context = zmq.Context()
        self.redis_client = redis.Redis(host='localhost', port=6379, db=0)
        self.kafka_producer = KafkaProducer(
            bootstrap_servers=['localhost:9092'],
            value_serializer=lambda v: json.dumps(v).encode('utf-8')
        )
        
        self.initialize_agent()
    
    def initialize_agent(self):
        """Initialize agent-specific components"""
        # Create ZMQ socket for receiving messages
        self.receiver_socket = self.zmq_context.socket(zmq.PULL)
        self.receiver_socket.bind(f"tcp://*:{5550 + list(AgentType).index(self.agent_type)}")
        
        # Create ZMQ socket for sending messages
        self.sender_socket = self.zmq_context.socket(zmq.PUSH)
        
        logging.info(f"Agent {self.agent_id} initialized")
    
    async def start(self):
        """Start the agent"""
        self.is_running = True
        
        # Start message handling
        asyncio.create_task(self.handle_messages())
        
        # Start agent-specific tasks
        asyncio.create_task(self.run_agent_tasks())
        
        logging.info(f"Agent {self.agent_id} started")
    
    async def stop(self):
        """Stop the agent"""
        self.is_running = False
        self.receiver_socket.close()
        self.sender_socket.close()
        logging.info(f"Agent {self.agent_id} stopped")
    
    async def send_message(self, receiver_id: str, message_type: MessageType, 
                          content: Dict[str, Any], priority: int = 1, 
                          correlation_id: Optional[str] = None):
        """Send message to another agent"""
        try:
            message = AgentMessage(
                message_id=str(uuid.uuid4()),
                sender_id=self.agent_id,
                receiver_id=receiver_id,
                message_type=message_type,
                content=content,
                timestamp=datetime.now(),
                priority=priority,
                correlation_id=correlation_id
            )
            
            # Send via ZMQ
            self.sender_socket.connect(f"tcp://localhost:{5550 + list(AgentType).index(AgentType(receiver_id.split('_')[0] + '_' + receiver_id.split('_')[1]))}")
            self.sender_socket.send_string(json.dumps(asdict(message)))
            
            # Also send via Kafka for persistence
            self.kafka_producer.send('agent_messages', value=asdict(message))
            
            logging.info(f"Message sent from {self.agent_id} to {receiver_id}")
            
        except Exception as e:
            logging.error(f"Failed to send message: {e}")
    
    async def handle_messages(self):
        """Handle incoming messages"""
        while self.is_running:
            try:
                # Receive message via ZMQ
                message_data = self.receiver_socket.recv_string(zmq.NOBLOCK)
                message = AgentMessage(**json.loads(message_data))
                
                # Process message
                await self.process_message(message)
                
            except zmq.Again:
                await asyncio.sleep(0.1)
            except Exception as e:
                logging.error(f"Error handling message: {e}")
                await asyncio.sleep(1)
    
    async def process_message(self, message: AgentMessage):
        """Process incoming message"""
        try:
            if message.message_type == MessageType.DATA_REQUEST:
                await self.handle_data_request(message)
            elif message.message_type == MessageType.ANALYSIS_REQUEST:
                await self.handle_analysis_request(message)
            elif message.message_type == MessageType.THREAT_ALERT:
                await self.handle_threat_alert(message)
            elif message.message_type == MessageType.COORDINATION:
                await self.handle_coordination(message)
            elif message.message_type == MessageType.TASK_ASSIGNMENT:
                await self.handle_task_assignment(message)
            
        except Exception as e:
            logging.error(f"Error processing message: {e}")
    
    @abstractmethod
    async def run_agent_tasks(self):
        """Run agent-specific tasks"""
        pass
    
    @abstractmethod
    async def handle_data_request(self, message: AgentMessage):
        """Handle data request"""
        pass
    
    @abstractmethod
    async def handle_analysis_request(self, message: AgentMessage):
        """Handle analysis request"""
        pass
    
    async def handle_threat_alert(self, message: AgentMessage):
        """Handle threat alert"""
        logging.info(f"Agent {self.agent_id} received threat alert: {message.content}")
    
    async def handle_coordination(self, message: AgentMessage):
        """Handle coordination message"""
        logging.info(f"Agent {self.agent_id} received coordination: {message.content}")
    
    async def handle_task_assignment(self, message: AgentMessage):
        """Handle task assignment"""
        logging.info(f"Agent {self.agent_id} received task: {message.content}")

class IntelligenceAnalystAgent(BaseAgent):
    """Intelligence Analyst Agent - Coordinates analysis and provides insights"""
    
    def __init__(self):
        capabilities = AgentCapabilities(
            can_analyze=['user_behavior', 'threat_patterns', 'network_activity'],
            can_collect=['intelligence_reports', 'analysis_results'],
            can_predict=['threat_evolution', 'user_behavior'],
            can_detect=['anomalies', 'patterns'],
            can_coordinate=True
        )
        super().__init__("intelligence_analyst_001", AgentType.INTELLIGENCE_ANALYST, capabilities)
        self.analysis_queue = []
        self.coordination_tasks = []
    
    async def run_agent_tasks(self):
        """Run intelligence analyst tasks"""
        while self.is_running:
            try:
                # Coordinate analysis tasks
                await self.coordinate_analysis()
                
                # Generate intelligence reports
                await self.generate_intelligence_reports()
                
                # Monitor threat landscape
                await self.monitor_threat_landscape()
                
                await asyncio.sleep(10)
                
            except Exception as e:
                logging.error(f"Intelligence analyst error: {e}")
                await asyncio.sleep(5)
    
    async def coordinate_analysis(self):
        """Coordinate analysis across all agents"""
        try:
            # Request data from data collector
            await self.send_message(
                "data_collector_001",
                MessageType.DATA_REQUEST,
                {"request_type": "recent_telegram_data", "limit": 1000}
            )
            
            # Request threat analysis from threat hunter
            await self.send_message(
                "threat_hunter_001",
                MessageType.ANALYSIS_REQUEST,
                {"analysis_type": "threat_assessment", "scope": "all_users"}
            )
            
            # Request network analysis
            await self.send_message(
                "network_analyst_001",
                MessageType.ANALYSIS_REQUEST,
                {"analysis_type": "network_mapping", "depth": "deep"}
            )
            
        except Exception as e:
            logging.error(f"Coordination error: {e}")
    
    async def generate_intelligence_reports(self):
        """Generate comprehensive intelligence reports"""
        try:
            # Collect data from all agents
            report_data = {
                'timestamp': datetime.now().isoformat(),
                'threat_level': await self.assess_overall_threat_level(),
                'key_findings': await self.extract_key_findings(),
                'recommendations': await self.generate_recommendations(),
                'trends': await self.analyze_trends()
            }
            
            # Store report
            self.knowledge_base['latest_report'] = report_data
            
            # Notify coordinator
            await self.send_message(
                "coordinator_001",
                MessageType.STATUS_UPDATE,
                {"status": "report_generated", "data": report_data}
            )
            
        except Exception as e:
            logging.error(f"Report generation error: {e}")
    
    async def monitor_threat_landscape(self):
        """Monitor overall threat landscape"""
        try:
            # Analyze threat indicators
            threat_indicators = await self.analyze_threat_indicators()
            
            if threat_indicators['risk_level'] > 0.7:
                # Send high-priority alert
                await self.send_message(
                    "coordinator_001",
                    MessageType.THREAT_ALERT,
                    {"alert_type": "high_risk", "indicators": threat_indicators},
                    priority=3
                )
            
        except Exception as e:
            logging.error(f"Threat monitoring error: {e}")
    
    async def handle_data_request(self, message: AgentMessage):
        """Handle data requests"""
        try:
            if message.content.get('request_type') == 'intelligence_report':
                # Provide latest intelligence report
                response_data = self.knowledge_base.get('latest_report', {})
                
                await self.send_message(
                    message.sender_id,
                    MessageType.DATA_RESPONSE,
                    {"data": response_data},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Data request handling error: {e}")
    
    async def handle_analysis_request(self, message: AgentMessage):
        """Handle analysis requests"""
        try:
            analysis_type = message.content.get('analysis_type')
            
            if analysis_type == 'comprehensive_analysis':
                # Perform comprehensive analysis
                analysis_result = await self.perform_comprehensive_analysis(message.content)
                
                await self.send_message(
                    message.sender_id,
                    MessageType.ANALYSIS_RESPONSE,
                    {"analysis": analysis_result},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Analysis request handling error: {e}")
    
    # Placeholder methods
    async def assess_overall_threat_level(self) -> float:
        return 0.5
    
    async def extract_key_findings(self) -> List[str]:
        return ["Sample finding 1", "Sample finding 2"]
    
    async def generate_recommendations(self) -> List[str]:
        return ["Recommendation 1", "Recommendation 2"]
    
    async def analyze_trends(self) -> Dict[str, Any]:
        return {"trend": "increasing"}
    
    async def analyze_threat_indicators(self) -> Dict[str, Any]:
        return {"risk_level": 0.3}
    
    async def perform_comprehensive_analysis(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        return {"analysis": "comprehensive"}

class ThreatHunterAgent(BaseAgent):
    """Threat Hunter Agent - Specialized in threat detection and analysis"""
    
    def __init__(self):
        capabilities = AgentCapabilities(
            can_analyze=['threat_patterns', 'malicious_behavior', 'attack_vectors'],
            can_collect=['threat_indicators', 'suspicious_activities'],
            can_predict=['threat_evolution', 'attack_probability'],
            can_detect=['malware', 'phishing', 'social_engineering']
        )
        super().__init__("threat_hunter_001", AgentType.THREAT_HUNTER, capabilities)
        self.threat_database = {}
        self.active_threats = []
    
    async def run_agent_tasks(self):
        """Run threat hunter tasks"""
        while self.is_running:
            try:
                # Hunt for threats
                await self.hunt_threats()
                
                # Analyze threat patterns
                await self.analyze_threat_patterns()
                
                # Update threat database
                await self.update_threat_database()
                
                await asyncio.sleep(5)
                
            except Exception as e:
                logging.error(f"Threat hunter error: {e}")
                await asyncio.sleep(5)
    
    async def hunt_threats(self):
        """Actively hunt for threats"""
        try:
            # Request recent data for analysis
            await self.send_message(
                "data_collector_001",
                MessageType.DATA_REQUEST,
                {"request_type": "suspicious_activities", "timeframe": "last_hour"}
            )
            
            # Analyze for threats
            threats = await self.detect_threats()
            
            for threat in threats:
                # Alert other agents
                await self.send_message(
                    "intelligence_analyst_001",
                    MessageType.THREAT_ALERT,
                    {"threat": threat, "severity": threat.get('severity', 'medium')},
                    priority=2
                )
                
                # Update active threats
                self.active_threats.append(threat)
            
        except Exception as e:
            logging.error(f"Threat hunting error: {e}")
    
    async def analyze_threat_patterns(self):
        """Analyze threat patterns"""
        try:
            # Analyze patterns in active threats
            patterns = await self.identify_threat_patterns()
            
            if patterns:
                # Share patterns with other agents
                await self.send_message(
                    "pattern_recognizer_001",
                    MessageType.ANALYSIS_REQUEST,
                    {"patterns": patterns, "analysis_type": "threat_patterns"}
                )
            
        except Exception as e:
            logging.error(f"Pattern analysis error: {e}")
    
    async def update_threat_database(self):
        """Update threat database"""
        try:
            # Update database with new threat information
            for threat in self.active_threats:
                threat_id = threat.get('id', str(uuid.uuid4()))
                self.threat_database[threat_id] = threat
            
            # Clean old threats
            current_time = datetime.now()
            self.active_threats = [
                threat for threat in self.active_threats
                if (current_time - threat.get('timestamp', current_time)).seconds < 3600
            ]
            
        except Exception as e:
            logging.error(f"Database update error: {e}")
    
    async def handle_data_request(self, message: AgentMessage):
        """Handle data requests"""
        try:
            if message.content.get('request_type') == 'threat_database':
                # Provide threat database
                await self.send_message(
                    message.sender_id,
                    MessageType.DATA_RESPONSE,
                    {"threats": list(self.threat_database.values())},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Data request handling error: {e}")
    
    async def handle_analysis_request(self, message: AgentMessage):
        """Handle analysis requests"""
        try:
            analysis_type = message.content.get('analysis_type')
            
            if analysis_type == 'threat_assessment':
                # Perform threat assessment
                assessment = await self.perform_threat_assessment(message.content)
                
                await self.send_message(
                    message.sender_id,
                    MessageType.ANALYSIS_RESPONSE,
                    {"assessment": assessment},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Analysis request handling error: {e}")
    
    # Placeholder methods
    async def detect_threats(self) -> List[Dict[str, Any]]:
        return [{"id": str(uuid.uuid4()), "type": "suspicious_activity", "severity": "medium"}]
    
    async def identify_threat_patterns(self) -> List[Dict[str, Any]]:
        return [{"pattern": "repeated_behavior", "confidence": 0.8}]
    
    async def perform_threat_assessment(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        return {"threat_level": "medium", "confidence": 0.7}

class NetworkAnalystAgent(BaseAgent):
    """Network Analyst Agent - Analyzes network connections and relationships"""
    
    def __init__(self):
        capabilities = AgentCapabilities(
            can_analyze=['network_topology', 'connection_patterns', 'influence_networks'],
            can_collect=['network_data', 'connection_metrics'],
            can_predict=['network_growth', 'influence_spread'],
            can_detect=['network_anomalies', 'suspicious_connections']
        )
        super().__init__("network_analyst_001", AgentType.NETWORK_ANALYST, capabilities)
        self.network_graph = {}
        self.connection_analysis = {}
    
    async def run_agent_tasks(self):
        """Run network analyst tasks"""
        while self.is_running:
            try:
                # Analyze network connections
                await self.analyze_network_connections()
                
                # Map influence networks
                await self.map_influence_networks()
                
                # Detect network anomalies
                await self.detect_network_anomalies()
                
                await asyncio.sleep(15)
                
            except Exception as e:
                logging.error(f"Network analyst error: {e}")
                await asyncio.sleep(5)
    
    async def analyze_network_connections(self):
        """Analyze network connections"""
        try:
            # Request network data
            await self.send_message(
                "data_collector_001",
                MessageType.DATA_REQUEST,
                {"request_type": "network_connections", "scope": "all"}
            )
            
            # Analyze connections
            analysis = await self.perform_connection_analysis()
            
            # Share with intelligence analyst
            await self.send_message(
                "intelligence_analyst_001",
                MessageType.ANALYSIS_RESPONSE,
                {"analysis_type": "network_connections", "results": analysis}
            )
            
        except Exception as e:
            logging.error(f"Network analysis error: {e}")
    
    async def map_influence_networks(self):
        """Map influence networks"""
        try:
            # Map influence patterns
            influence_map = await self.create_influence_map()
            
            # Identify key influencers
            key_influencers = await self.identify_key_influencers(influence_map)
            
            # Share with risk assessor
            await self.send_message(
                "risk_assessor_001",
                MessageType.ANALYSIS_REQUEST,
                {"analysis_type": "influence_assessment", "influencers": key_influencers}
            )
            
        except Exception as e:
            logging.error(f"Influence mapping error: {e}")
    
    async def detect_network_anomalies(self):
        """Detect network anomalies"""
        try:
            # Detect anomalies
            anomalies = await self.find_network_anomalies()
            
            if anomalies:
                # Alert threat hunter
                await self.send_message(
                    "threat_hunter_001",
                    MessageType.THREAT_ALERT,
                    {"anomalies": anomalies, "type": "network_anomaly"},
                    priority=2
                )
            
        except Exception as e:
            logging.error(f"Anomaly detection error: {e}")
    
    async def handle_data_request(self, message: AgentMessage):
        """Handle data requests"""
        try:
            if message.content.get('request_type') == 'network_analysis':
                # Provide network analysis
                await self.send_message(
                    message.sender_id,
                    MessageType.DATA_RESPONSE,
                    {"network_data": self.network_graph},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Data request handling error: {e}")
    
    async def handle_analysis_request(self, message: AgentMessage):
        """Handle analysis requests"""
        try:
            analysis_type = message.content.get('analysis_type')
            
            if analysis_type == 'network_mapping':
                # Perform network mapping
                mapping = await self.perform_network_mapping(message.content)
                
                await self.send_message(
                    message.sender_id,
                    MessageType.ANALYSIS_RESPONSE,
                    {"mapping": mapping},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Analysis request handling error: {e}")
    
    # Placeholder methods
    async def perform_connection_analysis(self) -> Dict[str, Any]:
        return {"connections": 100, "clusters": 5}
    
    async def create_influence_map(self) -> Dict[str, Any]:
        return {"influencers": 10, "reach": 1000}
    
    async def identify_key_influencers(self, influence_map: Dict[str, Any]) -> List[str]:
        return ["influencer1", "influencer2"]
    
    async def find_network_anomalies(self) -> List[Dict[str, Any]]:
        return [{"type": "unusual_connection", "severity": "low"}]
    
    async def perform_network_mapping(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        return {"nodes": 50, "edges": 200}

class CoordinatorAgent(BaseAgent):
    """Coordinator Agent - Orchestrates all other agents"""
    
    def __init__(self):
        capabilities = AgentCapabilities(
            can_analyze=['system_performance', 'agent_coordination'],
            can_collect=['agent_status', 'system_metrics'],
            can_predict=['workload_distribution', 'resource_needs'],
            can_detect=['agent_failures', 'coordination_issues'],
            can_coordinate=True
        )
        super().__init__("coordinator_001", AgentType.COORDINATOR, capabilities)
        self.agent_registry = {}
        self.task_queue = []
        self.system_metrics = {}
    
    async def run_agent_tasks(self):
        """Run coordinator tasks"""
        while self.is_running:
            try:
                # Monitor agent status
                await self.monitor_agent_status()
                
                # Distribute tasks
                await self.distribute_tasks()
                
                # Optimize system performance
                await self.optimize_system_performance()
                
                # Handle coordination requests
                await self.handle_coordination_requests()
                
                await asyncio.sleep(5)
                
            except Exception as e:
                logging.error(f"Coordinator error: {e}")
                await asyncio.sleep(5)
    
    async def monitor_agent_status(self):
        """Monitor status of all agents"""
        try:
            # Check agent health
            for agent_id, agent_info in self.agent_registry.items():
                # Send status check
                await self.send_message(
                    agent_id,
                    MessageType.STATUS_UPDATE,
                    {"request": "status_check"}
                )
            
        except Exception as e:
            logging.error(f"Agent monitoring error: {e}")
    
    async def distribute_tasks(self):
        """Distribute tasks to appropriate agents"""
        try:
            while self.task_queue:
                task = self.task_queue.pop(0)
                
                # Find best agent for task
                best_agent = await self.find_best_agent_for_task(task)
                
                if best_agent:
                    # Assign task
                    await self.send_message(
                        best_agent,
                        MessageType.TASK_ASSIGNMENT,
                        {"task": task}
                    )
                
        except Exception as e:
            logging.error(f"Task distribution error: {e}")
    
    async def optimize_system_performance(self):
        """Optimize overall system performance"""
        try:
            # Analyze system metrics
            performance_analysis = await self.analyze_system_performance()
            
            # Make optimization decisions
            optimizations = await self.determine_optimizations(performance_analysis)
            
            # Apply optimizations
            for optimization in optimizations:
                await self.apply_optimization(optimization)
            
        except Exception as e:
            logging.error(f"Performance optimization error: {e}")
    
    async def handle_coordination_requests(self):
        """Handle coordination requests from agents"""
        try:
            # Process coordination messages
            # This would handle requests for coordination between agents
            
        except Exception as e:
            logging.error(f"Coordination handling error: {e}")
    
    async def handle_data_request(self, message: AgentMessage):
        """Handle data requests"""
        try:
            if message.content.get('request_type') == 'system_status':
                # Provide system status
                status = {
                    'agents': list(self.agent_registry.keys()),
                    'tasks_pending': len(self.task_queue),
                    'system_metrics': self.system_metrics
                }
                
                await self.send_message(
                    message.sender_id,
                    MessageType.DATA_RESPONSE,
                    {"status": status},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Data request handling error: {e}")
    
    async def handle_analysis_request(self, message: AgentMessage):
        """Handle analysis requests"""
        try:
            analysis_type = message.content.get('analysis_type')
            
            if analysis_type == 'system_analysis':
                # Perform system analysis
                analysis = await self.perform_system_analysis()
                
                await self.send_message(
                    message.sender_id,
                    MessageType.ANALYSIS_RESPONSE,
                    {"analysis": analysis},
                    correlation_id=message.correlation_id
                )
            
        except Exception as e:
            logging.error(f"Analysis request handling error: {e}")
    
    # Placeholder methods
    async def find_best_agent_for_task(self, task: Dict[str, Any]) -> Optional[str]:
        return "intelligence_analyst_001"
    
    async def analyze_system_performance(self) -> Dict[str, Any]:
        return {"performance": "good"}
    
    async def determine_optimizations(self, analysis: Dict[str, Any]) -> List[Dict[str, Any]]:
        return [{"type": "load_balancing"}]
    
    async def apply_optimization(self, optimization: Dict[str, Any]):
        pass
    
    async def perform_system_analysis(self) -> Dict[str, Any]:
        return {"analysis": "system_healthy"}

class AIAgentsSystem:
    """Main system for managing all AI agents"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.agents = {}
        self.message_broker = None
        self.is_running = False
        
        self.initialize_agents()
    
    def initialize_agents(self):
        """Initialize all agents"""
        try:
            # Create agents
            self.agents['intelligence_analyst'] = IntelligenceAnalystAgent()
            self.agents['threat_hunter'] = ThreatHunterAgent()
            self.agents['network_analyst'] = NetworkAnalystAgent()
            self.agents['coordinator'] = CoordinatorAgent()
            
            logging.info("All agents initialized successfully")
            
        except Exception as e:
            logging.error(f"Failed to initialize agents: {e}")
            raise
    
    async def start_system(self):
        """Start the entire agent system"""
        try:
            self.is_running = True
            
            # Start all agents
            for agent in self.agents.values():
                await agent.start()
            
            logging.info("AI Agents System started successfully")
            
        except Exception as e:
            logging.error(f"Failed to start agent system: {e}")
            raise
    
    async def stop_system(self):
        """Stop the entire agent system"""
        try:
            self.is_running = False
            
            # Stop all agents
            for agent in self.agents.values():
                await agent.stop()
            
            logging.info("AI Agents System stopped successfully")
            
        except Exception as e:
            logging.error(f"Failed to stop agent system: {e}")
    
    async def send_system_message(self, target_agent: str, message_type: MessageType, 
                                 content: Dict[str, Any]):
        """Send message to specific agent"""
        try:
            if target_agent in self.agents:
                # This would send a message to the specific agent
                pass
            
        except Exception as e:
            logging.error(f"Failed to send system message: {e}")
    
    def get_system_status(self) -> Dict[str, Any]:
        """Get overall system status"""
        try:
            status = {
                'total_agents': len(self.agents),
                'active_agents': sum(1 for agent in self.agents.values() if agent.is_running),
                'system_running': self.is_running,
                'agents': {name: agent.agent_id for name, agent in self.agents.items()}
            }
            
            return status
            
        except Exception as e:
            logging.error(f"Failed to get system status: {e}")
            return {}

# Example usage
async def main():
    """Example usage of AI Agents System"""
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
    
    system = AIAgentsSystem(config)
    
    try:
        await system.start_system()
        
        # Keep running
        while True:
            await asyncio.sleep(1)
            
    except KeyboardInterrupt:
        await system.stop_system()

if __name__ == "__main__":
    asyncio.run(main())
