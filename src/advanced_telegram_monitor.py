#!/usr/bin/env python3
"""
Advanced Telegram Monitoring System with AI-Powered User Profiling
Continuously monitors chats, profiles users, and adapts strategies
"""

from telethon import TelegramClient, events, sync
from telethon.tl.functions.messages import GetHistoryRequest, GetFullChatRequest
from telethon.tl.functions.channels import JoinChannelRequest, GetFullChannelRequest
from telethon.tl.functions.contacts import ResolveUsernameRequest, GetContactsRequest
from telethon.tl.functions.users import GetFullUserRequest
from telethon.tl.types import User, Channel, Chat
import time
import random
import sqlite3
import json
import asyncio
import threading
import queue
from datetime import datetime, timedelta
import re
import hashlib
from collections import defaultdict, Counter
from chatgptapibypass import chatgpt
import requests
from urllib.parse import urlparse
import base64

class AdvancedUserProfiler:
    """Advanced user profiling and analysis system"""
    
    def __init__(self, db_connection):
        self.db = db_connection
        self.user_profiles = {}
        self.username_values = {}
        self.suspicious_patterns = []
        self.learning_data = {}
        
    async def profile_user_comprehensive(self, client, user_id, username=None):
        """Comprehensive user profiling including bio, profile pics, and activity"""
        try:
            # Get full user information
            user_entity = await client.get_entity(user_id)
            full_user = await client(GetFullUserRequest(user_entity))
            
            profile_data = {
                'user_id': user_id,
                'username': username or getattr(user_entity, 'username', ''),
                'first_name': getattr(user_entity, 'first_name', ''),
                'last_name': getattr(user_entity, 'last_name', ''),
                'phone': getattr(user_entity, 'phone', ''),
                'bio': getattr(full_user.full_user, 'about', ''),
                'profile_photo_id': getattr(full_user.full_user, 'profile_photo', None),
                'common_chats_count': getattr(full_user.full_user, 'common_chats_count', 0),
                'is_bot': getattr(user_entity, 'bot', False),
                'is_verified': getattr(user_entity, 'verified', False),
                'is_premium': getattr(user_entity, 'premium', False),
                'is_scam': getattr(user_entity, 'scam', False),
                'is_fake': getattr(user_entity, 'fake', False),
                'last_seen': getattr(user_entity, 'status', None),
                'profile_analyzed_at': datetime.now().isoformat()
            }
            
            # Analyze username value and patterns
            username_analysis = self.analyze_username_value(profile_data['username'])
            profile_data['username_analysis'] = username_analysis
            
            # Analyze bio for suspicious content
            bio_analysis = self.analyze_bio_content(profile_data['bio'])
            profile_data['bio_analysis'] = bio_analysis
            
            # Get user's recent activity across chats
            activity_analysis = await self.analyze_user_activity(client, user_id)
            profile_data['activity_analysis'] = activity_analysis
            
            # AI-powered personality and risk assessment
            ai_assessment = await self.ai_user_assessment(profile_data)
            profile_data['ai_assessment'] = ai_assessment
            
            # Store comprehensive profile
            self.store_user_profile(profile_data)
            
            return profile_data
            
        except Exception as e:
            print(f"Error profiling user {user_id}: {e}")
            return None
    
    def analyze_username_value(self, username):
        """Analyze username for value indicators and patterns"""
        if not username:
            return {'value_score': 0, 'patterns': [], 'risk_level': 'low'}
        
        analysis = {
            'value_score': 0,
            'patterns': [],
            'risk_level': 'low',
            'indicators': []
        }
        
        # Check for valuable username patterns
        valuable_patterns = [
            r'^[a-z]{1,3}$',  # Short usernames
            r'^[a-z]+\d{1,3}$',  # Name + numbers
            r'^[a-z]+[a-z]$',  # Double letters
            r'^[a-z]{4,8}$',  # Medium length clean names
        ]
        
        for pattern in valuable_patterns:
            if re.match(pattern, username.lower()):
                analysis['value_score'] += 10
                analysis['patterns'].append('valuable_format')
        
        # Check for business/crypto indicators
        business_keywords = ['crypto', 'btc', 'eth', 'trading', 'invest', 'money', 'cash', 'bank']
        for keyword in business_keywords:
            if keyword in username.lower():
                analysis['value_score'] += 15
                analysis['patterns'].append('business_related')
                analysis['indicators'].append(f'contains_{keyword}')
        
        # Check for suspicious patterns
        suspicious_patterns = [
            r'\d{4,}',  # Many numbers
            r'[a-z]{1}\d{3,}',  # Single letter + many numbers
            r'^[a-z]+\d+[a-z]+$',  # Mixed pattern
        ]
        
        for pattern in suspicious_patterns:
            if re.match(pattern, username.lower()):
                analysis['risk_level'] = 'medium'
                analysis['patterns'].append('suspicious_format')
        
        # Check for premium indicators
        if len(username) <= 3:
            analysis['value_score'] += 20
            analysis['patterns'].append('premium_short')
        
        return analysis
    
    def analyze_bio_content(self, bio):
        """Analyze bio content for suspicious or valuable information"""
        if not bio:
            return {'suspicious_score': 0, 'indicators': [], 'risk_level': 'low'}
        
        analysis = {
            'suspicious_score': 0,
            'indicators': [],
            'risk_level': 'low',
            'contact_info': [],
            'business_indicators': []
        }
        
        # Look for contact information
        phone_pattern = r'\+?[\d\s\-\(\)]{10,}'
        email_pattern = r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
        telegram_pattern = r'@[a-zA-Z0-9_]+'
        
        if re.search(phone_pattern, bio):
            analysis['contact_info'].append('phone')
            analysis['suspicious_score'] += 5
        
        if re.search(email_pattern, bio):
            analysis['contact_info'].append('email')
            analysis['suspicious_score'] += 3
        
        if re.search(telegram_pattern, bio):
            analysis['contact_info'].append('telegram')
            analysis['suspicious_score'] += 2
        
        # Look for business/crypto indicators
        business_keywords = [
            'crypto', 'bitcoin', 'btc', 'ethereum', 'eth', 'trading', 'invest',
            'money', 'cash', 'bank', 'loan', 'credit', 'debt', 'profit',
            'hustle', 'side', 'business', 'entrepreneur', 'ceo', 'founder'
        ]
        
        for keyword in business_keywords:
            if keyword.lower() in bio.lower():
                analysis['business_indicators'].append(keyword)
                analysis['suspicious_score'] += 2
        
        # Look for suspicious phrases
        suspicious_phrases = [
            'dm me', 'contact me', 'hit me up', 'let\'s talk', 'serious inquiries',
            'no time wasters', 'quick money', 'easy money', 'guaranteed',
            'risk free', 'investment opportunity', 'get rich quick'
        ]
        
        for phrase in suspicious_phrases:
            if phrase.lower() in bio.lower():
                analysis['indicators'].append(f'suspicious_phrase: {phrase}')
                analysis['suspicious_score'] += 5
        
        # Determine risk level
        if analysis['suspicious_score'] >= 15:
            analysis['risk_level'] = 'high'
        elif analysis['suspicious_score'] >= 8:
            analysis['risk_level'] = 'medium'
        
        return analysis
    
    async def analyze_user_activity(self, client, user_id):
        """Analyze user's activity patterns across different chats"""
        try:
            # Get user's dialogs to see what groups they're in
            dialogs = await client.get_dialogs()
            user_groups = []
            
            for dialog in dialogs:
                if hasattr(dialog.entity, 'participants_count'):
                    # This is a group/channel
                    try:
                        participants = await client.get_participants(dialog.entity, limit=1000)
                        if any(p.id == user_id for p in participants):
                            user_groups.append({
                                'title': getattr(dialog.entity, 'title', 'Unknown'),
                                'id': dialog.entity.id,
                                'type': 'channel' if isinstance(dialog.entity, Channel) else 'group'
                            })
                    except:
                        continue
            
            # Analyze message patterns from recent history
            message_patterns = await self.get_user_message_patterns(client, user_id)
            
            return {
                'groups_count': len(user_groups),
                'groups': user_groups,
                'message_patterns': message_patterns,
                'activity_level': self.calculate_activity_level(message_patterns)
            }
            
        except Exception as e:
            print(f"Error analyzing user activity: {e}")
            return {'error': str(e)}
    
    async def get_user_message_patterns(self, client, user_id):
        """Get and analyze user's message patterns"""
        patterns = {
            'total_messages': 0,
            'avg_message_length': 0,
            'common_words': [],
            'message_times': [],
            'topics': []
        }
        
        try:
            # Get recent messages from user across different chats
            dialogs = await client.get_dialogs()
            all_messages = []
            
            for dialog in dialogs[:10]:  # Check first 10 dialogs
                try:
                    messages = await client.get_messages(dialog.entity, limit=100, from_user=user_id)
                    all_messages.extend(messages)
                except:
                    continue
            
            if all_messages:
                patterns['total_messages'] = len(all_messages)
                
                # Analyze message content
                message_texts = [msg.text for msg in all_messages if msg.text]
                if message_texts:
                    patterns['avg_message_length'] = sum(len(text) for text in message_texts) / len(message_texts)
                    
                    # Common words analysis
                    all_words = []
                    for text in message_texts:
                        words = re.findall(r'\b\w+\b', text.lower())
                        all_words.extend(words)
                    
                    word_counts = Counter(all_words)
                    patterns['common_words'] = word_counts.most_common(10)
                    
                    # Message timing
                    patterns['message_times'] = [msg.date.hour for msg in all_messages if msg.date]
            
            return patterns
            
        except Exception as e:
            print(f"Error getting message patterns: {e}")
            return patterns
    
    def calculate_activity_level(self, message_patterns):
        """Calculate user's activity level"""
        if not message_patterns or message_patterns.get('total_messages', 0) == 0:
            return 'inactive'
        
        total_messages = message_patterns['total_messages']
        
        if total_messages >= 100:
            return 'very_active'
        elif total_messages >= 50:
            return 'active'
        elif total_messages >= 20:
            return 'moderate'
        else:
            return 'low'
    
    async def ai_user_assessment(self, profile_data):
        """AI-powered assessment of user's personality and risk level"""
        try:
            prompt = f"""
            Analyze this Telegram user profile and provide a comprehensive assessment:
            
            Username: {profile_data.get('username', 'N/A')}
            Bio: {profile_data.get('bio', 'N/A')}
            Username Analysis: {json.dumps(profile_data.get('username_analysis', {}), indent=2)}
            Bio Analysis: {json.dumps(profile_data.get('bio_analysis', {}), indent=2)}
            Activity Analysis: {json.dumps(profile_data.get('activity_analysis', {}), indent=2)}
            
            Provide assessment on:
            1. Personality traits and communication style
            2. Likely interests and expertise areas
            3. Potential value as a target (financial, information, network)
            4. Risk level for engagement (low/medium/high)
            5. Best approach for social engineering
            6. Potential illegal activities indicators
            7. Recommended conversation starters
            8. Red flags to watch for
            
            Format as JSON with detailed analysis.
            """
            
            response = chatgpt(prompt)
            if response:
                try:
                    return json.loads(response)
                except:
                    return {'raw_assessment': response}
            
        except Exception as e:
            print(f"AI assessment error: {e}")
        
        return {'error': 'AI assessment failed'}
    
    def store_user_profile(self, profile_data):
        """Store comprehensive user profile in database"""
        cursor = self.db.cursor()
        
        cursor.execute('''
            INSERT OR REPLACE INTO user_profiles (
                user_id, username, first_name, last_name, phone, bio,
                profile_photo_id, common_chats_count, is_bot, is_verified,
                is_premium, is_scam, is_fake, last_seen, profile_analyzed_at,
                username_analysis, bio_analysis, activity_analysis, ai_assessment
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            profile_data['user_id'],
            profile_data['username'],
            profile_data['first_name'],
            profile_data['last_name'],
            profile_data['phone'],
            profile_data['bio'],
            str(profile_data['profile_photo_id']),
            profile_data['common_chats_count'],
            profile_data['is_bot'],
            profile_data['is_verified'],
            profile_data['is_premium'],
            profile_data['is_scam'],
            profile_data['is_fake'],
            str(profile_data['last_seen']),
            profile_data['profile_analyzed_at'],
            json.dumps(profile_data['username_analysis']),
            json.dumps(profile_data['bio_analysis']),
            json.dumps(profile_data['activity_analysis']),
            json.dumps(profile_data['ai_assessment'])
        ))
        
        self.db.commit()

class ContinuousChatMonitor:
    """Continuous monitoring system for chats and new group discovery"""
    
    def __init__(self, db_connection, profiler):
        self.db = db_connection
        self.profiler = profiler
        self.monitored_chats = set()
        self.new_groups_found = []
        self.monitoring_active = True
        
    async def start_continuous_monitoring(self, client):
        """Start continuous monitoring of all chats"""
        print("Starting continuous chat monitoring...")
        
        # Get all current dialogs
        dialogs = await client.get_dialogs()
        
        for dialog in dialogs:
            if hasattr(dialog.entity, 'participants_count'):
                self.monitored_chats.add(dialog.entity.id)
                print(f"Monitoring chat: {getattr(dialog.entity, 'title', 'Unknown')}")
        
        # Start monitoring loop
        while self.monitoring_active:
            try:
                await self.monitor_cycle(client)
                await asyncio.sleep(300)  # 5 minute cycles
            except Exception as e:
                print(f"Monitoring cycle error: {e}")
                await asyncio.sleep(60)
    
    async def monitor_cycle(self, client):
        """Single monitoring cycle"""
        print(f"Monitoring cycle at {datetime.now()}")
        
        # Check for new messages in monitored chats
        for chat_id in list(self.monitored_chats):
            try:
                await self.check_new_messages(client, chat_id)
            except Exception as e:
                print(f"Error monitoring chat {chat_id}: {e}")
        
        # Discover new groups
        await self.discover_new_groups(client)
        
        # Profile new users
        await self.profile_new_users(client)
    
    async def check_new_messages(self, client, chat_id):
        """Check for new messages in a specific chat"""
        try:
            entity = await client.get_entity(chat_id)
            messages = await client.get_messages(entity, limit=10)
            
            for message in messages:
                if message.date > datetime.now() - timedelta(minutes=10):  # Last 10 minutes
                    await self.process_new_message(client, message, entity)
                    
        except Exception as e:
            print(f"Error checking messages for chat {chat_id}: {e}")
    
    async def process_new_message(self, client, message, chat_entity):
        """Process a new message and extract information"""
        try:
            user_id = message.sender_id
            username = getattr(message.sender, 'username', '') if message.sender else ''
            
            # Store message
            self.store_message(message, chat_entity)
            
            # Profile user if not already profiled recently
            if await self.should_profile_user(user_id):
                print(f"Profiling new user: {username} ({user_id})")
                await self.profiler.profile_user_comprehensive(client, user_id, username)
            
            # Check for suspicious content
            await self.check_suspicious_content(message, chat_entity)
            
        except Exception as e:
            print(f"Error processing message: {e}")
    
    async def should_profile_user(self, user_id):
        """Check if user should be profiled (not profiled recently)"""
        cursor = self.db.cursor()
        cursor.execute('''
            SELECT profile_analyzed_at FROM user_profiles 
            WHERE user_id = ? AND profile_analyzed_at > datetime('now', '-1 day')
        ''', (user_id,))
        
        result = cursor.fetchone()
        return result is None
    
    async def discover_new_groups(self, client):
        """Discover new groups based on keywords and user activity"""
        try:
            # Get trending keywords from current messages
            trending_keywords = await self.get_trending_keywords()
            
            for keyword in trending_keywords:
                await self.search_groups_by_keyword(client, keyword)
                
        except Exception as e:
            print(f"Error discovering new groups: {e}")
    
    async def get_trending_keywords(self):
        """Get trending keywords from recent messages"""
        cursor = self.db.cursor()
        cursor.execute('''
            SELECT message FROM raw_chat_data 
            WHERE created_at > datetime('now', '-1 hour')
            AND message != ''
        ''')
        
        messages = cursor.fetchall()
        all_words = []
        
        for (message,) in messages:
            words = re.findall(r'\b\w+\b', message.lower())
            all_words.extend(words)
        
        # Filter for relevant keywords
        relevant_keywords = []
        for word, count in Counter(all_words).most_common(20):
            if len(word) > 3 and count > 2:
                relevant_keywords.append(word)
        
        return relevant_keywords
    
    async def search_groups_by_keyword(self, client, keyword):
        """Search for groups containing a specific keyword"""
        try:
            # This is a simplified search - in practice, you'd need to use
            # Telegram's search functionality or known group lists
            search_terms = [
                f"{keyword}chat",
                f"{keyword}group",
                f"{keyword}community",
                f"{keyword}trading",
                f"{keyword}money"
            ]
            
            for term in search_terms:
                try:
                    # Try to resolve username
                    entity = await client(ResolveUsernameRequest(term))
                    if entity and entity.id not in self.monitored_chats:
                        print(f"Found new group: {term}")
                        self.new_groups_found.append({
                            'username': term,
                            'id': entity.id,
                            'title': getattr(entity, 'title', 'Unknown'),
                            'discovered_at': datetime.now().isoformat()
                        })
                        self.monitored_chats.add(entity.id)
                except:
                    continue
                    
        except Exception as e:
            print(f"Error searching groups for keyword {keyword}: {e}")
    
    async def profile_new_users(self, client):
        """Profile new users found in monitored chats"""
        try:
            # Get users who haven't been profiled recently
            cursor = self.db.cursor()
            cursor.execute('''
                SELECT DISTINCT user_id, username FROM raw_chat_data 
                WHERE user_id NOT IN (
                    SELECT user_id FROM user_profiles 
                    WHERE profile_analyzed_at > datetime('now', '-1 day')
                )
                AND created_at > datetime('now', '-1 hour')
                LIMIT 10
            ''')
            
            new_users = cursor.fetchall()
            
            for user_id, username in new_users:
                try:
                    await self.profiler.profile_user_comprehensive(client, user_id, username)
                    await asyncio.sleep(2)  # Rate limiting
                except Exception as e:
                    print(f"Error profiling user {user_id}: {e}")
                    
        except Exception as e:
            print(f"Error profiling new users: {e}")
    
    async def check_suspicious_content(self, message, chat_entity):
        """Check message content for suspicious patterns"""
        if not message.text:
            return
        
        suspicious_indicators = []
        
        # Check for illegal activity indicators
        illegal_keywords = [
            'drugs', 'cocaine', 'heroin', 'weed', 'marijuana',
            'weapons', 'guns', 'ammo', 'explosives',
            'fraud', 'scam', 'fake', 'counterfeit',
            'hack', 'hacking', 'malware', 'virus',
            'money laundering', 'tax evasion'
        ]
        
        for keyword in illegal_keywords:
            if keyword.lower() in message.text.lower():
                suspicious_indicators.append(f'illegal_keyword: {keyword}')
        
        # Check for financial scam indicators
        scam_phrases = [
            'guaranteed profit', 'risk free', 'get rich quick',
            'investment opportunity', 'double your money',
            'crypto investment', 'forex trading', 'binary options'
        ]
        
        for phrase in scam_phrases:
            if phrase.lower() in message.text.lower():
                suspicious_indicators.append(f'scam_phrase: {phrase}')
        
        if suspicious_indicators:
            self.store_suspicious_activity(message, chat_entity, suspicious_indicators)
    
    def store_message(self, message, chat_entity):
        """Store message in database"""
        cursor = self.db.cursor()
        cursor.execute('''
            INSERT INTO raw_chat_data (
                chat_id, chat_title, user_id, username, message, 
                response, timestamp, message_type, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            chat_entity.id,
            getattr(chat_entity, 'title', 'Unknown'),
            message.sender_id,
            getattr(message.sender, 'username', '') if message.sender else '',
            message.text or '',
            '',
            message.date.isoformat() if message.date else '',
            'telegram_message',
            datetime.now().isoformat()
        ))
        self.db.commit()
    
    def store_suspicious_activity(self, message, chat_entity, indicators):
        """Store suspicious activity for analysis"""
        cursor = self.db.cursor()
        cursor.execute('''
            INSERT INTO suspicious_activities (
                user_id, username, chat_id, chat_title, message,
                indicators, timestamp, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            message.sender_id,
            getattr(message.sender, 'username', '') if message.sender else '',
            chat_entity.id,
            getattr(chat_entity, 'title', 'Unknown'),
            message.text or '',
            json.dumps(indicators),
            message.date.isoformat() if message.date else '',
            datetime.now().isoformat()
        ))
        self.db.commit()
        
        print(f"SUSPICIOUS ACTIVITY DETECTED: {indicators}")

class AdaptiveAILearning:
    """AI system that learns and adapts from interactions"""
    
    def __init__(self, db_connection):
        self.db = db_connection
        self.interaction_history = []
        self.success_patterns = {}
        self.failure_patterns = {}
        self.learning_prompts = {}
        
    async def learn_from_interaction(self, context, response, outcome):
        """Learn from interaction outcomes"""
        learning_data = {
            'context': context,
            'response': response,
            'outcome': outcome,
            'timestamp': datetime.now().isoformat()
        }
        
        self.interaction_history.append(learning_data)
        
        # Update success/failure patterns
        if outcome['success']:
            self.update_success_patterns(context, response)
        else:
            self.update_failure_patterns(context, response)
        
        # Store learning data
        self.store_learning_data(learning_data)
        
        # Update AI prompts based on learning
        await self.update_ai_prompts()
    
    def update_success_patterns(self, context, response):
        """Update patterns that lead to successful interactions"""
        context_key = self.extract_context_key(context)
        
        if context_key not in self.success_patterns:
            self.success_patterns[context_key] = []
        
        self.success_patterns[context_key].append({
            'response': response,
            'timestamp': datetime.now().isoformat()
        })
    
    def update_failure_patterns(self, context, response):
        """Update patterns that lead to failed interactions"""
        context_key = self.extract_context_key(context)
        
        if context_key not in self.failure_patterns:
            self.failure_patterns[context_key] = []
        
        self.failure_patterns[context_key].append({
            'response': response,
            'timestamp': datetime.now().isoformat()
        })
    
    def extract_context_key(self, context):
        """Extract key features from context for pattern matching"""
        key_features = []
        
        if 'user_type' in context:
            key_features.append(f"user_type:{context['user_type']}")
        
        if 'group_type' in context:
            key_features.append(f"group_type:{context['group_type']}")
        
        if 'topic' in context:
            key_features.append(f"topic:{context['topic']}")
        
        return "|".join(key_features)
    
    async def update_ai_prompts(self):
        """Update AI prompts based on learning"""
        try:
            # Analyze success patterns
            success_analysis = self.analyze_success_patterns()
            
            # Generate new prompts based on learning
            new_prompts = await self.generate_adaptive_prompts(success_analysis)
            
            # Update stored prompts
            self.learning_prompts.update(new_prompts)
            
            # Store updated prompts
            self.store_updated_prompts()
            
        except Exception as e:
            print(f"Error updating AI prompts: {e}")
    
    def analyze_success_patterns(self):
        """Analyze patterns in successful interactions"""
        analysis = {
            'successful_contexts': {},
            'effective_responses': {},
            'common_elements': []
        }
        
        for context_key, patterns in self.success_patterns.items():
            if len(patterns) >= 3:  # Only analyze patterns with enough data
                analysis['successful_contexts'][context_key] = {
                    'count': len(patterns),
                    'recent_success_rate': self.calculate_recent_success_rate(context_key)
                }
        
        return analysis
    
    def calculate_recent_success_rate(self, context_key):
        """Calculate recent success rate for a context"""
        recent_cutoff = datetime.now() - timedelta(days=7)
        
        success_count = 0
        total_count = 0
        
        # Count recent successes
        if context_key in self.success_patterns:
            for pattern in self.success_patterns[context_key]:
                if datetime.fromisoformat(pattern['timestamp']) > recent_cutoff:
                    success_count += 1
                    total_count += 1
        
        # Count recent failures
        if context_key in self.failure_patterns:
            for pattern in self.failure_patterns[context_key]:
                if datetime.fromisoformat(pattern['timestamp']) > recent_cutoff:
                    total_count += 1
        
        return success_count / total_count if total_count > 0 else 0
    
    async def generate_adaptive_prompts(self, success_analysis):
        """Generate new prompts based on success analysis"""
        try:
            prompt = f"""
            Based on this analysis of successful interactions, create improved prompts:
            
            Success Analysis: {json.dumps(success_analysis, indent=2)}
            
            Create new prompts that:
            1. Leverage successful patterns
            2. Avoid failed approaches
            3. Adapt to different user types
            4. Improve response effectiveness
            5. Include dynamic elements based on context
            
            Generate prompts for:
            - User engagement
            - Conversation starters
            - Information gathering
            - Trust building
            - Social engineering
            
            Format as JSON with prompt categories and specific prompts.
            """
            
            response = chatgpt(prompt)
            if response:
                try:
                    return json.loads(response)
                except:
                    return {'adaptive_prompts': response}
            
        except Exception as e:
            print(f"Error generating adaptive prompts: {e}")
        
        return {}
    
    def store_learning_data(self, learning_data):
        """Store learning data in database"""
        cursor = self.db.cursor()
        cursor.execute('''
            INSERT INTO ai_learning_data (
                context, response, outcome, timestamp, created_at
            ) VALUES (?, ?, ?, ?, ?)
        ''', (
            json.dumps(learning_data['context']),
            learning_data['response'],
            json.dumps(learning_data['outcome']),
            learning_data['timestamp'],
            datetime.now().isoformat()
        ))
        self.db.commit()
    
    def store_updated_prompts(self):
        """Store updated prompts in database"""
        cursor = self.db.cursor()
        cursor.execute('''
            INSERT INTO adaptive_prompts (
                prompt_category, prompt_content, success_rate, 
                usage_count, last_updated, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        ''', (
            'adaptive',
            json.dumps(self.learning_prompts),
            0.0,  # Will be calculated
            0,
            datetime.now().isoformat(),
            datetime.now().isoformat()
        ))
        self.db.commit()

# Enhanced database setup
def setup_advanced_database():
    """Setup database with advanced tables"""
    conn = sqlite3.connect("advanced_telegram_monitor.db")
    cursor = conn.cursor()
    
    # User profiles table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS user_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT UNIQUE,
            username TEXT,
            first_name TEXT,
            last_name TEXT,
            phone TEXT,
            bio TEXT,
            profile_photo_id TEXT,
            common_chats_count INTEGER,
            is_bot BOOLEAN,
            is_verified BOOLEAN,
            is_premium BOOLEAN,
            is_scam BOOLEAN,
            is_fake BOOLEAN,
            last_seen TEXT,
            profile_analyzed_at TIMESTAMP,
            username_analysis TEXT,
            bio_analysis TEXT,
            activity_analysis TEXT,
            ai_assessment TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    
    # Suspicious activities table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS suspicious_activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            username TEXT,
            chat_id TEXT,
            chat_title TEXT,
            message TEXT,
            indicators TEXT,
            timestamp TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    
    # AI learning data table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS ai_learning_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            context TEXT,
            response TEXT,
            outcome TEXT,
            timestamp TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    
    # Adaptive prompts table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS adaptive_prompts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            prompt_category TEXT,
            prompt_content TEXT,
            success_rate REAL,
            usage_count INTEGER,
            last_updated TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    
    # Raw chat data table (enhanced)
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS raw_chat_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id TEXT,
            chat_title TEXT,
            user_id TEXT,
            username TEXT,
            message TEXT,
            response TEXT,
            timestamp TEXT,
            message_type TEXT,
            sentiment_score REAL,
            suspicious_score REAL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    
    conn.commit()
    return conn

# Main execution
async def main():
    """Main execution function"""
    print("Starting Advanced Telegram Monitor...")
    
    # Setup database
    db = setup_advanced_database()
    
    # Initialize components
    profiler = AdvancedUserProfiler(db)
    monitor = ContinuousChatMonitor(db, profiler)
    ai_learning = AdaptiveAILearning(db)
    
    # Initialize Telegram client (you'll need to configure this)
    # client = TelegramClient('session', api_id, api_hash)
    # await client.start()
    
    # Start monitoring
    # await monitor.start_continuous_monitoring(client)
    
    print("Advanced monitoring system initialized")

if __name__ == "__main__":
    asyncio.run(main())
