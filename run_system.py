#!/usr/bin/env python3

import asyncio
import sys
import os
import logging
from datetime import datetime

sys.path.append(os.path.join(os.path.dirname(__file__), 'src'))

from src.intelligent_ai_conversation_system import IntelligentAIConversationSystem
from src.ai_api_integration import AIAPIIntegration
from src.advanced_analytics_engine import AdvancedAnalyticsEngine
from src.ai_agents_system import AIAgentsSystem
from src.real_time_streaming_engine import RealTimeStreamingEngine

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('ai_intelligence_system.log'),
        logging.StreamHandler()
    ]
)

class AIIntelligenceSystem:
    def __init__(self):
        self.config = {
            'openai': {
                'api_key': os.getenv('OPENAI_API_KEY', 'your-openai-key'),
                'rate_limit': 60
            },
            'anthropic': {
                'api_key': os.getenv('ANTHROPIC_API_KEY', 'your-anthropic-key'),
                'rate_limit': 60
            },
            'google': {
                'api_key': os.getenv('GOOGLE_API_KEY', 'your-google-key'),
                'rate_limit': 60
            },
            'huggingface': {
                'api_key': os.getenv('HUGGINGFACE_API_KEY', 'your-huggingface-key'),
                'rate_limit': 60
            },
            'cohere': {
                'api_key': os.getenv('COHERE_API_KEY', 'your-cohere-key'),
                'rate_limit': 60
            },
            'together': {
                'api_key': os.getenv('TOGETHER_API_KEY', 'your-together-key'),
                'rate_limit': 60
            },
            'kafka': {
                'bootstrap_servers': ['localhost:9092']
            },
            'redis': {
                'host': 'localhost',
                'port': 6379,
                'db': 0
            },
            'chatgpt': {
                'email': os.getenv('CHATGPT_EMAIL', 'your-email@example.com'),
                'password': os.getenv('CHATGPT_PASSWORD', 'your-password')
            },
            'claude': {
                'email': os.getenv('CLAUDE_EMAIL', 'your-email@example.com'),
                'password': os.getenv('CLAUDE_PASSWORD', 'your-password')
            }
        }
        
        self.systems = {}
        self.is_running = False
    
    async def initialize_systems(self):
        try:
            logging.info("Initializing AI Intelligence System...")
            
            self.systems['ai_integration'] = AIAPIIntegration(self.config)
            self.systems['analytics_engine'] = AdvancedAnalyticsEngine(self.config)
            self.systems['ai_agents'] = AIAgentsSystem(self.config)
            self.systems['streaming_engine'] = RealTimeStreamingEngine(self.config)
            self.systems['conversation_system'] = IntelligentAIConversationSystem(self.config)
            
            logging.info("All systems initialized successfully")
            
        except Exception as e:
            logging.error(f"System initialization failed: {e}")
            raise
    
    async def start_system(self):
        try:
            self.is_running = True
            logging.info("Starting AI Intelligence System...")
            
            await self.systems['ai_agents'].start_system()
            await self.systems['streaming_engine'].start_streaming()
            await self.systems['conversation_system'].start_conversation_system()
            
            logging.info("AI Intelligence System started successfully")
            
        except Exception as e:
            logging.error(f"Failed to start system: {e}")
            raise
    
    async def stop_system(self):
        try:
            self.is_running = False
            logging.info("Stopping AI Intelligence System...")
            
            await self.systems['conversation_system'].stop_conversation_system()
            await self.systems['streaming_engine'].stop_streaming()
            await self.systems['ai_agents'].stop_system()
            
            logging.info("AI Intelligence System stopped successfully")
            
        except Exception as e:
            logging.error(f"Failed to stop system: {e}")
    
    async def run(self):
        try:
            await self.initialize_systems()
            await self.start_system()
            
            logging.info("System is running. Press Ctrl+C to stop.")
            
            while self.is_running:
                await asyncio.sleep(1)
                
        except KeyboardInterrupt:
            logging.info("Received interrupt signal")
        except Exception as e:
            logging.error(f"System error: {e}")
        finally:
            await self.stop_system()

async def main():
    system = AIIntelligenceSystem()
    await system.run()

if __name__ == "__main__":
    asyncio.run(main())
