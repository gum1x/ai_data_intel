#!/usr/bin/env python3
"""
Enhanced Data Collector - Advanced information gathering capabilities
"""

import asyncio
import json
import re
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Set
from dataclasses import dataclass
from enum import Enum
import logging
from collections import defaultdict, Counter
import aiohttp
import requests
from urllib.parse import urlparse, parse_qs
import networkx as nx

# Telegram
from telethon import TelegramClient
from telethon.tl.functions.users import GetFullUserRequest
from telethon.tl.functions.channels import GetFullChannelRequest
from telethon.tl.functions.contacts import ResolveUsernameRequest
from telethon.tl.types import User, Channel, Chat, Message

class DataCollectionType(Enum):
    USER_PROFILE = "user_profile"
    MESSAGE_CONTENT = "message_content"
    NETWORK_ANALYSIS = "network_analysis"
    BEHAVIORAL_PATTERNS = "behavioral_patterns"
    EXTERNAL_LINKS = "external_links"
    MEDIA_ANALYSIS = "media_analysis"
    TEMPORAL_PATTERNS = "temporal_patterns"

@dataclass
class EnhancedUserProfile:
    """Enhanced user profile with comprehensive data"""
    user_id: str
    username: str
    first_name: str
    last_name: str
    phone: str
    bio: str
    profile_photo_id: str
    is_verified: bool
    is_premium: bool
    is_bot: bool
    is_scam: bool
    is_fake: bool
    common_chats_count: int
    last_seen: str
    created_at: datetime
    # Enhanced data
    groups: List[Dict[str, Any]]
    contacts: List[Dict[str, Any]]
    forwarded_messages: List[Dict[str, Any]]
    media_files: List[Dict[str, Any]]
    links_shared: List[Dict[str, Any]]
    mentions: List[str]
    hashtags: List[str]
    crypto_addresses: List[str]
    phone_numbers: List[str]
    emails: List[str]
    social_links: List[str]
    behavioral_patterns: Dict[str, Any]
    communication_style: Dict[str, Any]
    activity_timeline: List[Dict[str, Any]]
    network_connections: List[Dict[str, Any]]
    suspicious_indicators: List[str]

class EnhancedDataCollector:
    """Enhanced data collector with advanced information gathering"""
    
    def __init__(self):
        self.collection_cache = {}
        self.pattern_cache = {}
        self.network_graph = nx.Graph()
        self.user_interactions = defaultdict(list)
        self.temporal_patterns = defaultdict(list)
        self.initialize_patterns()
    
    def initialize_patterns(self):
        """Initialize regex patterns for data extraction"""
        self.patterns = {
            'crypto_addresses': {
                'bitcoin': r'[13][a-km-zA-HJ-NP-Z1-9]{25,34}',
                'ethereum': r'0x[a-fA-F0-9]{40}',
                'litecoin': r'[LM3][a-km-zA-HJ-NP-Z1-9]{26,33}',
                'dogecoin': r'D{1}[5-9A-HJ-NP-U]{1}[1-9A-HJ-NP-Za-km-z]{32}',
                'monero': r'4[0-9AB][1-9A-HJ-NP-Za-km-z]{93}'
            },
            'phone_numbers': [
                r'\+?[\d\s\-\(\)]{10,}',
                r'\+?1?[-.\s]?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}',
                r'\+?[1-9]\d{1,14}'
            ],
            'emails': [
                r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}',
                r'[a-zA-Z0-9._%+-]+\+[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
            ],
            'social_links': [
                r'https?://(?:www\.)?(?:twitter\.com|x\.com)/[a-zA-Z0-9_]+',
                r'https?://(?:www\.)?instagram\.com/[a-zA-Z0-9_.]+',
                r'https?://(?:www\.)?facebook\.com/[a-zA-Z0-9.]+',
                r'https?://(?:www\.)?linkedin\.com/in/[a-zA-Z0-9-]+',
                r'https?://(?:www\.)?tiktok\.com/@[a-zA-Z0-9_.]+',
                r'https?://(?:www\.)?youtube\.com/(?:c/|channel/|user/)?[a-zA-Z0-9_-]+',
                r'https?://(?:www\.)?discord\.gg/[a-zA-Z0-9]+',
                r'https?://(?:www\.)?telegram\.me/[a-zA-Z0-9_]+',
                r'https?://t\.me/[a-zA-Z0-9_]+'
            ],
            'mentions': [
                r'@[a-zA-Z0-9_]+',
                r'@[a-zA-Z0-9_]{5,32}'
            ],
            'hashtags': [
                r'#[a-zA-Z0-9_]+',
                r'#[a-zA-Z0-9_]{2,100}'
            ],
            'urls': [
                r'https?://[^\s<>"{}|\\^`\[\]]+',
                r'www\.[^\s<>"{}|\\^`\[\]]+\.[a-zA-Z]{2,}'
            ]
        }
    
    async def collect_comprehensive_user_data(self, client: TelegramClient, user: User, 
                                            messages: List[Message]) -> EnhancedUserProfile:
        """Collect comprehensive user data with enhanced information gathering"""
        try:
            # Basic user info
            user_id = str(user.id)
            username = getattr(user, 'username', '')
            
            # Get full user information
            full_user = await client(GetFullUserRequest(user))
            
            # Extract basic profile data
            profile_data = {
                'user_id': user_id,
                'username': username,
                'first_name': getattr(user, 'first_name', ''),
                'last_name': getattr(user, 'last_name', ''),
                'phone': getattr(user, 'phone', ''),
                'bio': getattr(full_user.full_user, 'about', ''),
                'profile_photo_id': str(getattr(full_user.full_user, 'profile_photo', '')),
                'is_verified': getattr(user, 'verified', False),
                'is_premium': getattr(user, 'premium', False),
                'is_bot': getattr(user, 'bot', False),
                'is_scam': getattr(user, 'scam', False),
                'is_fake': getattr(user, 'fake', False),
                'common_chats_count': getattr(full_user.full_user, 'common_chats_count', 0),
                'last_seen': str(getattr(user, 'status', '')),
                'created_at': datetime.now()
            }
            
            # Enhanced data collection
            enhanced_data = await self.collect_enhanced_data(client, user, messages)
            
            # Combine all data
            profile_data.update(enhanced_data)
            
            return EnhancedUserProfile(**profile_data)
            
        except Exception as e:
            logging.error(f"Enhanced data collection error: {e}")
            return self.create_empty_profile(user)
    
    async def collect_enhanced_data(self, client: TelegramClient, user: User, 
                                  messages: List[Message]) -> Dict[str, Any]:
        """Collect enhanced data from various sources"""
        enhanced_data = {
            'groups': [],
            'contacts': [],
            'forwarded_messages': [],
            'media_files': [],
            'links_shared': [],
            'mentions': [],
            'hashtags': [],
            'crypto_addresses': [],
            'phone_numbers': [],
            'emails': [],
            'social_links': [],
            'behavioral_patterns': {},
            'communication_style': {},
            'activity_timeline': [],
            'network_connections': [],
            'suspicious_indicators': []
        }
        
        try:
            # Analyze messages for patterns
            message_analysis = await self.analyze_messages(messages)
            enhanced_data.update(message_analysis)
            
            # Get user's groups
            enhanced_data['groups'] = await self.get_user_groups(client, user)
            
            # Get user's contacts
            enhanced_data['contacts'] = await self.get_user_contacts(client, user)
            
            # Analyze behavioral patterns
            enhanced_data['behavioral_patterns'] = await self.analyze_behavioral_patterns(messages)
            
            # Analyze communication style
            enhanced_data['communication_style'] = await self.analyze_communication_style(messages)
            
            # Create activity timeline
            enhanced_data['activity_timeline'] = await self.create_activity_timeline(messages)
            
            # Analyze network connections
            enhanced_data['network_connections'] = await self.analyze_network_connections(client, user)
            
            # Identify suspicious indicators
            enhanced_data['suspicious_indicators'] = await self.identify_suspicious_indicators(
                enhanced_data, messages
            )
            
        except Exception as e:
            logging.error(f"Enhanced data collection error: {e}")
        
        return enhanced_data
    
    async def analyze_messages(self, messages: List[Message]) -> Dict[str, Any]:
        """Analyze messages for various patterns and data"""
        analysis = {
            'forwarded_messages': [],
            'media_files': [],
            'links_shared': [],
            'mentions': [],
            'hashtags': [],
            'crypto_addresses': [],
            'phone_numbers': [],
            'emails': [],
            'social_links': []
        }
        
        for message in messages:
            if not message.text:
                continue
            
            text = message.text
            
            # Extract forwarded messages
            if message.fwd_from:
                analysis['forwarded_messages'].append({
                    'original_sender': str(message.fwd_from.from_id),
                    'forward_date': message.fwd_from.date.isoformat() if message.fwd_from.date else '',
                    'forwarded_by': str(message.sender_id),
                    'message_id': message.id
                })
            
            # Extract media files
            if message.media:
                analysis['media_files'].append({
                    'media_type': str(type(message.media).__name__),
                    'file_size': getattr(message.media, 'size', 0),
                    'mime_type': getattr(message.media, 'mime_type', ''),
                    'message_id': message.id
                })
            
            # Extract crypto addresses
            for crypto_type, pattern in self.patterns['crypto_addresses'].items():
                matches = re.findall(pattern, text)
                for match in matches:
                    analysis['crypto_addresses'].append({
                        'type': crypto_type,
                        'address': match,
                        'message_id': message.id
                    })
            
            # Extract phone numbers
            for pattern in self.patterns['phone_numbers']:
                matches = re.findall(pattern, text)
                for match in matches:
                    analysis['phone_numbers'].append({
                        'number': match,
                        'message_id': message.id
                    })
            
            # Extract emails
            for pattern in self.patterns['emails']:
                matches = re.findall(pattern, text)
                for match in matches:
                    analysis['emails'].append({
                        'email': match,
                        'message_id': message.id
                    })
            
            # Extract social links
            for pattern in self.patterns['social_links']:
                matches = re.findall(pattern, text)
                for match in matches:
                    analysis['social_links'].append({
                        'url': match,
                        'message_id': message.id
                    })
            
            # Extract mentions
            for pattern in self.patterns['mentions']:
                matches = re.findall(pattern, text)
                analysis['mentions'].extend(matches)
            
            # Extract hashtags
            for pattern in self.patterns['hashtags']:
                matches = re.findall(pattern, text)
                analysis['hashtags'].extend(matches)
            
            # Extract all URLs
            for pattern in self.patterns['urls']:
                matches = re.findall(pattern, text)
                for match in matches:
                    analysis['links_shared'].append({
                        'url': match,
                        'message_id': message.id
                    })
        
        # Remove duplicates
        analysis['mentions'] = list(set(analysis['mentions']))
        analysis['hashtags'] = list(set(analysis['hashtags']))
        
        return analysis
    
    async def get_user_groups(self, client: TelegramClient, user: User) -> List[Dict[str, Any]]:
        """Get user's group memberships"""
        groups = []
        
        try:
            # Get user's dialogs
            dialogs = await client.get_dialogs()
            
            for dialog in dialogs:
                if hasattr(dialog.entity, 'participants_count'):
                    # This is a group/channel
                    try:
                        # Check if user is a member
                        participants = await client.get_participants(dialog.entity, limit=1000)
                        if any(p.id == user.id for p in participants):
                            groups.append({
                                'group_id': str(dialog.entity.id),
                                'group_title': getattr(dialog.entity, 'title', ''),
                                'group_username': getattr(dialog.entity, 'username', ''),
                                'member_count': getattr(dialog.entity, 'participants_count', 0),
                                'is_channel': isinstance(dialog.entity, Channel),
                                'is_verified': getattr(dialog.entity, 'verified', False)
                            })
                    except:
                        continue
            
        except Exception as e:
            logging.error(f"Error getting user groups: {e}")
        
        return groups
    
    async def get_user_contacts(self, client: TelegramClient, user: User) -> List[Dict[str, Any]]:
        """Get user's contacts"""
        contacts = []
        
        try:
            # Get user's contacts
            contact_list = await client(GetContactsRequest(hash=0))
            
            for contact in contact_list.users:
                contacts.append({
                    'user_id': str(contact.id),
                    'username': getattr(contact, 'username', ''),
                    'first_name': getattr(contact, 'first_name', ''),
                    'last_name': getattr(contact, 'last_name', ''),
                    'phone': getattr(contact, 'phone', ''),
                    'is_verified': getattr(contact, 'verified', False)
                })
            
        except Exception as e:
            logging.error(f"Error getting user contacts: {e}")
        
        return contacts
    
    async def analyze_behavioral_patterns(self, messages: List[Message]) -> Dict[str, Any]:
        """Analyze behavioral patterns from messages"""
        patterns = {
            'message_frequency': 0,
            'avg_message_length': 0,
            'response_time_patterns': [],
            'activity_hours': [],
            'language_patterns': {},
            'emotion_patterns': {},
            'topic_patterns': {},
            'interaction_patterns': {}
        }
        
        if not messages:
            return patterns
        
        # Message frequency
        patterns['message_frequency'] = len(messages)
        
        # Average message length
        total_length = sum(len(msg.text or '') for msg in messages)
        patterns['avg_message_length'] = total_length / len(messages)
        
        # Activity hours
        for message in messages:
            if message.date:
                patterns['activity_hours'].append(message.date.hour)
        
        # Language patterns
        all_text = ' '.join([msg.text or '' for msg in messages])
        patterns['language_patterns'] = {
            'total_characters': len(all_text),
            'total_words': len(all_text.split()),
            'avg_words_per_message': len(all_text.split()) / len(messages),
            'uppercase_ratio': sum(1 for c in all_text if c.isupper()) / max(len(all_text), 1),
            'digit_ratio': sum(1 for c in all_text if c.isdigit()) / max(len(all_text), 1),
            'special_char_ratio': sum(1 for c in all_text if not c.isalnum() and c != ' ') / max(len(all_text), 1)
        }
        
        return patterns
    
    async def analyze_communication_style(self, messages: List[Message]) -> Dict[str, Any]:
        """Analyze communication style"""
        style = {
            'formality_level': 'casual',
            'aggressiveness': 0.0,
            'friendliness': 0.0,
            'professionalism': 0.0,
            'common_phrases': [],
            'emoji_usage': 0.0,
            'punctuation_patterns': {}
        }
        
        if not messages:
            return style
        
        all_text = ' '.join([msg.text or '' for msg in messages])
        
        # Emoji usage
        emoji_count = sum(1 for c in all_text if ord(c) > 127)
        style['emoji_usage'] = emoji_count / max(len(all_text), 1)
        
        # Punctuation patterns
        style['punctuation_patterns'] = {
            'exclamation_ratio': all_text.count('!') / max(len(all_text), 1),
            'question_ratio': all_text.count('?') / max(len(all_text), 1),
            'period_ratio': all_text.count('.') / max(len(all_text), 1),
            'comma_ratio': all_text.count(',') / max(len(all_text), 1)
        }
        
        # Aggressiveness indicators
        aggressive_words = ['hate', 'kill', 'destroy', 'attack', 'fight', 'war']
        aggressive_count = sum(1 for word in aggressive_words if word in all_text.lower())
        style['aggressiveness'] = aggressive_count / max(len(all_text.split()), 1)
        
        # Friendliness indicators
        friendly_words = ['love', 'like', 'happy', 'good', 'great', 'awesome', 'thanks']
        friendly_count = sum(1 for word in friendly_words if word in all_text.lower())
        style['friendliness'] = friendly_count / max(len(all_text.split()), 1)
        
        return style
    
    async def create_activity_timeline(self, messages: List[Message]) -> List[Dict[str, Any]]:
        """Create activity timeline from messages"""
        timeline = []
        
        for message in messages:
            if message.date:
                timeline.append({
                    'timestamp': message.date.isoformat(),
                    'message_id': message.id,
                    'message_length': len(message.text or ''),
                    'has_media': bool(message.media),
                    'is_forwarded': bool(message.fwd_from)
                })
        
        # Sort by timestamp
        timeline.sort(key=lambda x: x['timestamp'])
        
        return timeline
    
    async def analyze_network_connections(self, client: TelegramClient, user: User) -> List[Dict[str, Any]]:
        """Analyze network connections"""
        connections = []
        
        try:
            # Get common chats
            full_user = await client(GetFullUserRequest(user))
            common_chats_count = getattr(full_user.full_user, 'common_chats_count', 0)
            
            connections.append({
                'type': 'common_chats',
                'count': common_chats_count,
                'description': f'User shares {common_chats_count} common chats'
            })
            
        except Exception as e:
            logging.error(f"Error analyzing network connections: {e}")
        
        return connections
    
    async def identify_suspicious_indicators(self, enhanced_data: Dict[str, Any], 
                                          messages: List[Message]) -> List[str]:
        """Identify suspicious indicators"""
        indicators = []
        
        # Check for multiple crypto addresses
        if len(enhanced_data.get('crypto_addresses', [])) > 3:
            indicators.append('multiple_crypto_addresses')
        
        # Check for multiple phone numbers
        if len(enhanced_data.get('phone_numbers', [])) > 2:
            indicators.append('multiple_phone_numbers')
        
        # Check for multiple social links
        if len(enhanced_data.get('social_links', [])) > 5:
            indicators.append('excessive_social_links')
        
        # Check for forwarded messages
        if len(enhanced_data.get('forwarded_messages', [])) > 10:
            indicators.append('excessive_forwarding')
        
        # Check for media files
        if len(enhanced_data.get('media_files', [])) > 20:
            indicators.append('excessive_media_sharing')
        
        # Check behavioral patterns
        behavioral = enhanced_data.get('behavioral_patterns', {})
        if behavioral.get('avg_message_length', 0) > 500:
            indicators.append('unusually_long_messages')
        
        if behavioral.get('message_frequency', 0) > 100:
            indicators.append('high_message_frequency')
        
        # Check communication style
        style = enhanced_data.get('communication_style', {})
        if style.get('aggressiveness', 0) > 0.1:
            indicators.append('aggressive_communication')
        
        if style.get('emoji_usage', 0) > 0.2:
            indicators.append('excessive_emoji_usage')
        
        return indicators
    
    def create_empty_profile(self, user: User) -> EnhancedUserProfile:
        """Create empty profile when error occurs"""
        return EnhancedUserProfile(
            user_id=str(user.id),
            username=getattr(user, 'username', ''),
            first_name=getattr(user, 'first_name', ''),
            last_name=getattr(user, 'last_name', ''),
            phone='',
            bio='',
            profile_photo_id='',
            is_verified=False,
            is_premium=False,
            is_bot=False,
            is_scam=False,
            is_fake=False,
            common_chats_count=0,
            last_seen='',
            created_at=datetime.now(),
            groups=[],
            contacts=[],
            forwarded_messages=[],
            media_files=[],
            links_shared=[],
            mentions=[],
            hashtags=[],
            crypto_addresses=[],
            phone_numbers=[],
            emails=[],
            social_links=[],
            behavioral_patterns={},
            communication_style={},
            activity_timeline=[],
            network_connections=[],
            suspicious_indicators=[]
        )

# Example usage
async def main():
    """Example usage of EnhancedDataCollector"""
    collector = EnhancedDataCollector()
    
    # This would be used with actual Telegram client and user data
    print("Enhanced Data Collector initialized")

if __name__ == "__main__":
    asyncio.run(main())
