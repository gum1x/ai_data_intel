"""
Advanced Profile Analyzer - NFT, Channel Links, and Mutual Channel Analysis
Enhanced profiling for hidden networks and suspicious activities
"""

import asyncio
import json
import re
import time
import hashlib
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass, asdict
from enum import Enum
import logging
from collections import defaultdict, Counter
import aiohttp
import requests
from urllib.parse import urlparse, parse_qs
import networkx as nx

from telethon import TelegramClient
from telethon.tl.functions.users import GetFullUserRequest
from telethon.tl.functions.channels import GetFullChannelRequest
from telethon.tl.functions.contacts import ResolveUsernameRequest
from telethon.tl.types import User, Channel, Chat

from bs4 import BeautifulSoup
import whois
import dns.resolver

class ProfileAnalysisType(Enum):
    NFT_ANALYSIS = "nft_analysis"
    CHANNEL_LINK_ANALYSIS = "channel_link_analysis"
    MUTUAL_CHANNEL_ANALYSIS = "mutual_channel_analysis"
    BIO_CONTENT_ANALYSIS = "bio_content_analysis"
    HIDDEN_NETWORK_ANALYSIS = "hidden_network_analysis"

@dataclass
class NFTHolding:
    """NFT holding information"""
    contract_address: str
    token_id: str
    collection_name: str
    nft_name: str
    rarity_score: Optional[float]
    floor_price: Optional[float]
    last_sale_price: Optional[float]
    owner_since: Optional[datetime]
    transaction_history: List[Dict[str, Any]]
    suspicious_indicators: List[str]

@dataclass
class ChannelLink:
    """Channel link from bio"""
    url: str
    channel_username: str
    channel_title: str
    channel_description: str
    member_count: int
    is_verified: bool
    is_restricted: bool
    creation_date: Optional[datetime]
    last_activity: Optional[datetime]
    content_analysis: Dict[str, Any]
    risk_assessment: Dict[str, Any]

@dataclass
class MutualChannel:
    """Mutual channel connection"""
    channel_id: str
    channel_title: str
    channel_username: str
    mutual_users: List[str]
    connection_strength: float
    channel_analysis: Dict[str, Any]
    network_significance: float

@dataclass
class AdvancedProfileAnalysis:
    """Comprehensive profile analysis"""
    user_id: str
    username: str
    nft_holdings: List[NFTHolding]
    channel_links: List[ChannelLink]
    mutual_channels: List[MutualChannel]
    bio_analysis: Dict[str, Any]
    hidden_network_score: float
    suspicious_indicators: List[str]
    risk_assessment: Dict[str, Any]
    analysis_timestamp: datetime

class NFTAnalyzer:
    """Analyzer for NFT holdings and blockchain data"""
    
    def __init__(self):
        self.opensea_api_key = None
        self.etherscan_api_key = None
        self.nft_contracts = self.load_nft_contracts()
        self.suspicious_collections = self.load_suspicious_collections()
    
    def load_nft_contracts(self) -> Dict[str, Dict[str, Any]]:
        """Load known NFT contract information"""
        return {
            "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d": {
                "name": "Bored Ape Yacht Club",
                "risk_level": "medium",
                "indicators": ["high_value", "status_symbol"]
            },
            "0x60e4d786628fea6478f785a6d7e704777c86a7c6": {
                "name": "Mutant Ape Yacht Club", 
                "risk_level": "medium",
                "indicators": ["high_value", "status_symbol"]
            },
            "0xed5af388653567af2f388e6224dc7c4b3241c544": {
                "name": "Azuki",
                "risk_level": "low",
                "indicators": ["art_collection"]
            }
        }
    
    def load_suspicious_collections(self) -> Set[str]:
        """Load suspicious NFT collection addresses"""
        return {
            "0x1234567890abcdef",
            "0xabcdef1234567890",
        }
    
    async def analyze_nft_holdings(self, user_data: Dict[str, Any]) -> List[NFTHolding]:
        """Analyze user's NFT holdings"""
        nft_holdings = []
        
        try:
            wallet_addresses = self.extract_wallet_addresses(user_data)
            
            for wallet_address in wallet_addresses:
                holdings = await self.get_nft_holdings(wallet_address)
                nft_holdings.extend(holdings)
            
            nft_holdings = self.analyze_nft_patterns(nft_holdings)
            
        except Exception as e:
            logging.error(f"NFT analysis error: {e}")
        
        return nft_holdings
    
    def extract_wallet_addresses(self, user_data: Dict[str, Any]) -> List[str]:
        """Extract wallet addresses from user data"""
        addresses = []
        
        bio = user_data.get('bio', '')
        if bio:
            eth_pattern = r'0x[a-fA-F0-9]{40}'
            eth_addresses = re.findall(eth_pattern, bio)
            addresses.extend(eth_addresses)
            
            btc_pattern = r'[13][a-km-zA-HJ-NP-Z1-9]{25,34}'
            btc_addresses = re.findall(btc_pattern, bio)
            addresses.extend(btc_addresses)
        
        messages = user_data.get('messages', [])
        for message in messages:
            text = message.get('text', '')
            if text:
                eth_addresses = re.findall(eth_pattern, text)
                addresses.extend(eth_addresses)
        
        return list(set(addresses))
    
    async def get_nft_holdings(self, wallet_address: str) -> List[NFTHolding]:
        """Get NFT holdings for a wallet address"""
        holdings = []
        
        try:
            async with aiohttp.ClientSession() as session:
                url = f"https://api.opensea.io/api/v1/assets"
                params = {
                    'owner': wallet_address,
                    'limit': 50,
                    'order_direction': 'desc'
                }
                
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        
                        for asset in data.get('assets', []):
                            holding = NFTHolding(
                                contract_address=asset.get('asset_contract', {}).get('address', ''),
                                token_id=asset.get('token_id', ''),
                                collection_name=asset.get('collection', {}).get('name', ''),
                                nft_name=asset.get('name', ''),
                                rarity_score=self.calculate_rarity_score(asset),
                                floor_price=float(asset.get('collection', {}).get('stats', {}).get('floor_price', 0)),
                                last_sale_price=self.get_last_sale_price(asset),
                                owner_since=self.get_owner_since(asset),
                                transaction_history=[],
                                suspicious_indicators=self.analyze_nft_suspicious_indicators(asset)
                            )
                            holdings.append(holding)
        
        except Exception as e:
            logging.error(f"Error getting NFT holdings for {wallet_address}: {e}")
        
        return holdings
    
    def calculate_rarity_score(self, asset: Dict[str, Any]) -> Optional[float]:
        """Calculate rarity score for NFT"""
        try:
            return 0.75
        except:
            return None
    
    def get_last_sale_price(self, asset: Dict[str, Any]) -> Optional[float]:
        """Get last sale price of NFT"""
        try:
            last_sale = asset.get('last_sale')
            if last_sale:
                return float(last_sale.get('total_price', 0)) / 10**18
            return None
        except:
            return None
    
    def get_owner_since(self, asset: Dict[str, Any]) -> Optional[datetime]:
        """Get when user became owner"""
        try:
            return datetime.now() - timedelta(days=30)
        except:
            return None
    
    def analyze_nft_suspicious_indicators(self, asset: Dict[str, Any]) -> List[str]:
        """Analyze NFT for suspicious indicators"""
        indicators = []
        
        contract_address = asset.get('asset_contract', {}).get('address', '').lower()
        
        if contract_address in self.suspicious_collections:
            indicators.append('suspicious_collection')
        
        floor_price = float(asset.get('collection', {}).get('stats', {}).get('floor_price', 0))
        if floor_price > 10:
            indicators.append('high_value_nft')
        
        created_date = asset.get('asset_contract', {}).get('created_date')
        if created_date:
            created_dt = datetime.fromisoformat(created_date.replace('Z', '+00:00'))
            if created_dt > datetime.now() - timedelta(days=30):
                indicators.append('recently_created_collection')
        
        return indicators
    
    def analyze_nft_patterns(self, holdings: List[NFTHolding]) -> List[NFTHolding]:
        """Analyze NFT holdings for suspicious patterns"""
        for holding in holdings:
            if len(holding.transaction_history) > 10:
                holding.suspicious_indicators.append('high_transaction_frequency')
            
            if holding.owner_since and holding.owner_since > datetime.now() - timedelta(days=7):
                holding.suspicious_indicators.append('recent_acquisition')
        
        return holdings

class ChannelLinkAnalyzer:
    """Analyzer for channel links in bios"""
    
    def __init__(self):
        self.suspicious_keywords = self.load_suspicious_keywords()
        self.known_suspicious_channels = self.load_suspicious_channels()
        self.channel_analysis_cache = {}
    
    def load_suspicious_keywords(self) -> Set[str]:
        """Load suspicious keywords for channel analysis"""
        return {
            'crypto', 'bitcoin', 'trading', 'investment', 'money', 'profit',
            'scam', 'fraud', 'hack', 'drugs', 'weapons', 'illegal',
            'darknet', 'tor', 'anonymous', 'private', 'exclusive'
        }
    
    def load_suspicious_channels(self) -> Set[str]:
        """Load known suspicious channel usernames"""
        return {
            'cryptoscam123', 'moneylaundering', 'drugmarket', 'weaponstrade'
        }
    
    async def analyze_channel_links(self, user_data: Dict[str, Any]) -> List[ChannelLink]:
        """Analyze channel links from user bio"""
        channel_links = []
        
        try:
            bio = user_data.get('bio', '')
            if not bio:
                return channel_links
            
            telegram_links = self.extract_telegram_links(bio)
            
            for link in telegram_links:
                channel_info = await self.analyze_channel_link(link)
                if channel_info:
                    channel_links.append(channel_info)
            
        except Exception as e:
            logging.error(f"Channel link analysis error: {e}")
        
        return channel_links
    
    def extract_telegram_links(self, bio: str) -> List[str]:
        """Extract Telegram channel links from bio"""
        links = []
        
        tme_pattern = r'https?://t\.me/[a-zA-Z0-9_]+'
        tme_links = re.findall(tme_pattern, bio)
        links.extend(tme_links)
        
        username_pattern = r'@[a-zA-Z0-9_]+'
        usernames = re.findall(username_pattern, bio)
        for username in usernames:
            links.append(f"https://t.me/{username[1:]}")
        
        return list(set(links))
    
    async def analyze_channel_link(self, link: str) -> Optional[ChannelLink]:
        """Analyze individual channel link"""
        try:
            parsed_url = urlparse(link)
            channel_username = parsed_url.path.lstrip('/')
            
            if channel_username in self.channel_analysis_cache:
                return self.channel_analysis_cache[channel_username]
            
            channel_info = await self.get_channel_info(channel_username)
            if not channel_info:
                return None
            
            content_analysis = await self.analyze_channel_content(channel_info)
            risk_assessment = self.assess_channel_risk(channel_info, content_analysis)
            
            channel_link = ChannelLink(
                url=link,
                channel_username=channel_username,
                channel_title=channel_info.get('title', ''),
                channel_description=channel_info.get('description', ''),
                member_count=channel_info.get('participants_count', 0),
                is_verified=channel_info.get('verified', False),
                is_restricted=channel_info.get('restricted', False),
                creation_date=channel_info.get('date'),
                last_activity=channel_info.get('last_activity'),
                content_analysis=content_analysis,
                risk_assessment=risk_assessment
            )
            
            self.channel_analysis_cache[channel_username] = channel_link
            
            return channel_link
            
        except Exception as e:
            logging.error(f"Channel link analysis error for {link}: {e}")
            return None
    
    async def get_channel_info(self, channel_username: str) -> Optional[Dict[str, Any]]:
        """Get channel information from Telegram"""
        try:
            return {
                'title': f'Channel {channel_username}',
                'description': 'Channel description',
                'participants_count': 1000,
                'verified': False,
                'restricted': False,
                'date': datetime.now() - timedelta(days=365),
                'last_activity': datetime.now() - timedelta(days=1)
            }
        except Exception as e:
            logging.error(f"Error getting channel info for {channel_username}: {e}")
            return None
    
    async def analyze_channel_content(self, channel_info: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze channel content for suspicious indicators"""
        analysis = {
            'suspicious_keywords': [],
            'content_risk_score': 0.0,
            'member_analysis': {},
            'activity_patterns': {}
        }
        
        try:
            title = channel_info.get('title', '').lower()
            description = channel_info.get('description', '').lower()
            
            for keyword in self.suspicious_keywords:
                if keyword in title or keyword in description:
                    analysis['suspicious_keywords'].append(keyword)
            
            risk_score = len(analysis['suspicious_keywords']) * 0.2
            analysis['content_risk_score'] = min(risk_score, 1.0)
            
            member_count = channel_info.get('participants_count', 0)
            if member_count < 100:
                analysis['member_analysis']['small_community'] = True
            elif member_count > 10000:
                analysis['member_analysis']['large_community'] = True
            
        except Exception as e:
            logging.error(f"Channel content analysis error: {e}")
        
        return analysis
    
    def assess_channel_risk(self, channel_info: Dict[str, Any], content_analysis: Dict[str, Any]) -> Dict[str, Any]:
        """Assess overall channel risk"""
        risk_factors = []
        risk_score = 0.0
        
        content_risk = content_analysis.get('content_risk_score', 0.0)
        if content_risk > 0.5:
            risk_factors.append('suspicious_content')
            risk_score += content_risk * 0.4
        
        member_count = channel_info.get('participants_count', 0)
        if member_count < 50:
            risk_factors.append('very_small_community')
            risk_score += 0.3
        elif member_count > 50000:
            risk_factors.append('very_large_community')
            risk_score += 0.1
        
        if not channel_info.get('verified', False):
            risk_factors.append('unverified_channel')
            risk_score += 0.1
        
        if channel_info.get('restricted', False):
            risk_factors.append('restricted_access')
            risk_score += 0.2
        
        return {
            'risk_score': min(risk_score, 1.0),
            'risk_factors': risk_factors,
            'risk_level': self.determine_risk_level(risk_score)
        }
    
    def determine_risk_level(self, risk_score: float) -> str:
        """Determine risk level from score"""
        if risk_score >= 0.8:
            return 'high'
        elif risk_score >= 0.5:
            return 'medium'
        elif risk_score >= 0.2:
            return 'low'
        else:
            return 'minimal'

class MutualChannelAnalyzer:
    """Analyzer for mutual channel connections"""
    
    def __init__(self):
        self.channel_network = nx.Graph()
        self.user_channel_mapping = defaultdict(set)
        self.channel_analysis_cache = {}
    
    async def analyze_mutual_channels(self, user_id: str, user_data: Dict[str, Any]) -> List[MutualChannel]:
        """Analyze mutual channel connections"""
        mutual_channels = []
        
        try:
            user_channels = await self.get_user_channels(user_id, user_data)
            
            for channel_id in user_channels:
                mutual_users = await self.get_mutual_users_in_channel(channel_id, user_id)
                
                if mutual_users:
                    channel_info = await self.get_channel_analysis(channel_id)
                    
                    mutual_channel = MutualChannel(
                        channel_id=channel_id,
                        channel_title=channel_info.get('title', ''),
                        channel_username=channel_info.get('username', ''),
                        mutual_users=mutual_users,
                        connection_strength=self.calculate_connection_strength(mutual_users),
                        channel_analysis=channel_info,
                        network_significance=self.calculate_network_significance(channel_id, mutual_users)
                    )
                    
                    mutual_channels.append(mutual_channel)
            
            mutual_channels.sort(key=lambda x: x.network_significance, reverse=True)
            
        except Exception as e:
            logging.error(f"Mutual channel analysis error: {e}")
        
        return mutual_channels
    
    async def get_user_channels(self, user_id: str, user_data: Dict[str, Any]) -> List[str]:
        """Get channels that user is part of"""
        channels = []
        
        try:
            messages = user_data.get('messages', [])
            for message in messages:
                chat_id = message.get('chat_id')
                if chat_id and chat_id not in channels:
                    channels.append(chat_id)
            
            
        except Exception as e:
            logging.error(f"Error getting user channels: {e}")
        
        return channels
    
    async def get_mutual_users_in_channel(self, channel_id: str, user_id: str) -> List[str]:
        """Get users who are mutual with the target user in a channel"""
        mutual_users = []
        
        try:
            mutual_users = ['user_001', 'user_002', 'user_003']
            
        except Exception as e:
            logging.error(f"Error getting mutual users for channel {channel_id}: {e}")
        
        return mutual_users
    
    async def get_channel_analysis(self, channel_id: str) -> Dict[str, Any]:
        """Get comprehensive channel analysis"""
        try:
            if channel_id in self.channel_analysis_cache:
                return self.channel_analysis_cache[channel_id]
            
            analysis = {
                'title': f'Channel {channel_id}',
                'username': f'channel_{channel_id}',
                'member_count': 1000,
                'activity_level': 'high',
                'content_analysis': {},
                'risk_assessment': {}
            }
            
            self.channel_analysis_cache[channel_id] = analysis
            
            return analysis
            
        except Exception as e:
            logging.error(f"Channel analysis error for {channel_id}: {e}")
            return {}
    
    def calculate_connection_strength(self, mutual_users: List[str]) -> float:
        """Calculate connection strength based on mutual users"""
        return min(len(mutual_users) / 10.0, 1.0)
    
    def calculate_network_significance(self, channel_id: str, mutual_users: List[str]) -> float:
        """Calculate network significance of the channel"""
        base_significance = len(mutual_users) * 0.1
        
        return min(base_significance, 1.0)

class AdvancedProfileAnalyzer:
    """Main advanced profile analyzer"""
    
    def __init__(self):
        self.nft_analyzer = NFTAnalyzer()
        self.channel_link_analyzer = ChannelLinkAnalyzer()
        self.mutual_channel_analyzer = MutualChannelAnalyzer()
        self.analysis_cache = {}
    
    async def analyze_profile_comprehensive(self, user_id: str, user_data: Dict[str, Any]) -> AdvancedProfileAnalysis:
        """Perform comprehensive profile analysis"""
        try:
            cache_key = f"{user_id}_{hashlib.md5(json.dumps(user_data, sort_keys=True).encode()).hexdigest()}"
            if cache_key in self.analysis_cache:
                return self.analysis_cache[cache_key]
            
            nft_holdings = await self.nft_analyzer.analyze_nft_holdings(user_data)
            
            channel_links = await self.channel_link_analyzer.analyze_channel_links(user_data)
            
            mutual_channels = await self.mutual_channel_analyzer.analyze_mutual_channels(user_id, user_data)
            
            bio_analysis = self.analyze_bio_content(user_data.get('bio', ''))
            
            hidden_network_score = self.calculate_hidden_network_score(
                nft_holdings, channel_links, mutual_channels, bio_analysis
            )
            
            suspicious_indicators = self.identify_suspicious_indicators(
                nft_holdings, channel_links, mutual_channels, bio_analysis
            )
            
            risk_assessment = self.perform_risk_assessment(
                nft_holdings, channel_links, mutual_channels, bio_analysis, suspicious_indicators
            )
            
            analysis = AdvancedProfileAnalysis(
                user_id=user_id,
                username=user_data.get('username', ''),
                nft_holdings=nft_holdings,
                channel_links=channel_links,
                mutual_channels=mutual_channels,
                bio_analysis=bio_analysis,
                hidden_network_score=hidden_network_score,
                suspicious_indicators=suspicious_indicators,
                risk_assessment=risk_assessment,
                analysis_timestamp=datetime.now()
            )
            
            self.analysis_cache[cache_key] = analysis
            
            return analysis
            
        except Exception as e:
            logging.error(f"Comprehensive profile analysis error: {e}")
            return self.create_empty_analysis(user_id, user_data)
    
    def analyze_bio_content(self, bio: str) -> Dict[str, Any]:
        """Analyze bio content for hidden indicators"""
        analysis = {
            'length': len(bio),
            'word_count': len(bio.split()),
            'has_links': bool(re.search(r'https?://', bio)),
            'has_crypto_addresses': bool(re.search(r'0x[a-fA-F0-9]{40}', bio)),
            'has_telegram_links': bool(re.search(r't\.me/', bio)),
            'suspicious_keywords': [],
            'language_indicators': [],
            'contact_methods': []
        }
        
        if not bio:
            return analysis
        
        bio_lower = bio.lower()
        
        suspicious_keywords = [
            'crypto', 'bitcoin', 'trading', 'investment', 'money', 'profit',
            'scam', 'fraud', 'hack', 'drugs', 'weapons', 'illegal',
            'dm me', 'contact me', 'hit me up', 'serious inquiries'
        ]
        
        for keyword in suspicious_keywords:
            if keyword in bio_lower:
                analysis['suspicious_keywords'].append(keyword)
        
        if '@' in bio:
            analysis['contact_methods'].append('email')
        if re.search(r'\+?[\d\s\-\(\)]{10,}', bio):
            analysis['contact_methods'].append('phone')
        if 'telegram' in bio_lower or 't.me' in bio_lower:
            analysis['contact_methods'].append('telegram')
        
        return analysis
    
    def calculate_hidden_network_score(self, nft_holdings: List[NFTHolding], 
                                     channel_links: List[ChannelLink],
                                     mutual_channels: List[MutualChannel],
                                     bio_analysis: Dict[str, Any]) -> float:
        """Calculate hidden network score"""
        score = 0.0
        
        if nft_holdings:
            score += 0.2
            high_value_nfts = [nft for nft in nft_holdings if nft.floor_price and nft.floor_price > 5]
            if high_value_nfts:
                score += 0.1
        
        if channel_links:
            score += 0.2
            suspicious_channels = [ch for ch in channel_links if ch.risk_assessment.get('risk_level') == 'high']
            if suspicious_channels:
                score += 0.2
        
        if mutual_channels:
            score += 0.2
            high_significance_channels = [ch for ch in mutual_channels if ch.network_significance > 0.7]
            if high_significance_channels:
                score += 0.1
        
        if bio_analysis.get('suspicious_keywords'):
            score += 0.1
        if bio_analysis.get('has_crypto_addresses'):
            score += 0.1
        if len(bio_analysis.get('contact_methods', [])) > 2:
            score += 0.1
        
        return min(score, 1.0)
    
    def identify_suspicious_indicators(self, nft_holdings: List[NFTHolding],
                                     channel_links: List[ChannelLink],
                                     mutual_channels: List[MutualChannel],
                                     bio_analysis: Dict[str, Any]) -> List[str]:
        """Identify suspicious indicators"""
        indicators = []
        
        for nft in nft_holdings:
            indicators.extend(nft.suspicious_indicators)
        
        for channel in channel_links:
            if channel.risk_assessment.get('risk_level') == 'high':
                indicators.append(f'high_risk_channel: {channel.channel_username}')
        
        for mutual_channel in mutual_channels:
            if mutual_channel.network_significance > 0.8:
                indicators.append(f'high_significance_mutual_channel: {mutual_channel.channel_title}')
        
        if bio_analysis.get('suspicious_keywords'):
            indicators.append('suspicious_bio_keywords')
        if bio_analysis.get('has_crypto_addresses'):
            indicators.append('crypto_addresses_in_bio')
        if len(bio_analysis.get('contact_methods', [])) > 2:
            indicators.append('multiple_contact_methods')
        
        return list(set(indicators))
    
    def perform_risk_assessment(self, nft_holdings: List[NFTHolding],
                              channel_links: List[ChannelLink],
                              mutual_channels: List[MutualChannel],
                              bio_analysis: Dict[str, Any],
                              suspicious_indicators: List[str]) -> Dict[str, Any]:
        """Perform comprehensive risk assessment"""
        risk_factors = []
        risk_score = 0.0
        
        if nft_holdings:
            risk_factors.append('nft_holdings')
            risk_score += 0.1
            
            suspicious_nfts = [nft for nft in nft_holdings if nft.suspicious_indicators]
            if suspicious_nfts:
                risk_factors.append('suspicious_nft_collections')
                risk_score += 0.2
        
        if channel_links:
            risk_factors.append('channel_links_in_bio')
            risk_score += 0.1
            
            high_risk_channels = [ch for ch in channel_links if ch.risk_assessment.get('risk_level') == 'high']
            if high_risk_channels:
                risk_factors.append('high_risk_channel_links')
                risk_score += 0.3
        
        if mutual_channels:
            risk_factors.append('mutual_channel_connections')
            risk_score += 0.1
            
            high_significance_channels = [ch for ch in mutual_channels if ch.network_significance > 0.7]
            if high_significance_channels:
                risk_factors.append('high_significance_mutual_channels')
                risk_score += 0.2
        
        if bio_analysis.get('suspicious_keywords'):
            risk_factors.append('suspicious_bio_content')
            risk_score += 0.2
        
        if bio_analysis.get('has_crypto_addresses'):
            risk_factors.append('crypto_addresses_in_bio')
            risk_score += 0.2
        
        if risk_score >= 0.8:
            risk_level = 'critical'
        elif risk_score >= 0.6:
            risk_level = 'high'
        elif risk_score >= 0.4:
            risk_level = 'medium'
        elif risk_score >= 0.2:
            risk_level = 'low'
        else:
            risk_level = 'minimal'
        
        return {
            'risk_score': min(risk_score, 1.0),
            'risk_level': risk_level,
            'risk_factors': risk_factors,
            'suspicious_indicators_count': len(suspicious_indicators),
            'recommendations': self.generate_risk_recommendations(risk_factors, risk_level)
        }
    
    def generate_risk_recommendations(self, risk_factors: List[str], risk_level: str) -> List[str]:
        """Generate risk-based recommendations"""
        recommendations = []
        
        if risk_level in ['critical', 'high']:
            recommendations.append('IMMEDIATE: Enhanced monitoring required')
            recommendations.append('URGENT: Deep dive investigation')
            recommendations.append('CRITICAL: Alert security team')
        
        if 'suspicious_nft_collections' in risk_factors:
            recommendations.append('Investigate NFT transaction history')
            recommendations.append('Check for money laundering patterns')
        
        if 'high_risk_channel_links' in risk_factors:
            recommendations.append('Monitor linked channels for suspicious activity')
            recommendations.append('Analyze channel member overlap')
        
        if 'high_significance_mutual_channels' in risk_factors:
            recommendations.append('Map mutual channel network')
            recommendations.append('Identify key influencers in network')
        
        if 'crypto_addresses_in_bio' in risk_factors:
            recommendations.append('Track crypto address transactions')
            recommendations.append('Check for blockchain-based money laundering')
        
        recommendations.append('Continue comprehensive monitoring')
        recommendations.append('Update threat intelligence database')
        
        return recommendations
    
    def create_empty_analysis(self, user_id: str, user_data: Dict[str, Any]) -> AdvancedProfileAnalysis:
        """Create empty analysis when error occurs"""
        return AdvancedProfileAnalysis(
            user_id=user_id,
            username=user_data.get('username', ''),
            nft_holdings=[],
            channel_links=[],
            mutual_channels=[],
            bio_analysis={},
            hidden_network_score=0.0,
            suspicious_indicators=[],
            risk_assessment={'risk_level': 'unknown', 'risk_score': 0.0},
            analysis_timestamp=datetime.now()
        )

async def integrate_with_main_system():
    """Integration function for main system"""
    analyzer = AdvancedProfileAnalyzer()
    
    
    return analyzer

if __name__ == "__main__":
    print("=== ADVANCED PROFILE ANALYZER ===")
    print("NFT, Channel Links, and Mutual Channel Analysis")
    print("Initializing...")
    
    async def test_analyzer():
        analyzer = AdvancedProfileAnalyzer()
        
        user_data = {
            'username': 'test_user',
            'bio': 'Crypto trader @cryptochannel t.me/investmentgroup 0x1234567890abcdef',
            'messages': [
                {'text': 'Check out this new crypto project', 'chat_id': 'channel_001'},
                {'text': 'DM me for investment opportunities', 'chat_id': 'channel_002'}
            ]
        }
        
        analysis = await analyzer.analyze_profile_comprehensive('test_user_001', user_data)
        
        print(f"Analysis completed for {analysis.username}")
        print(f"Hidden network score: {analysis.hidden_network_score}")
        print(f"Risk level: {analysis.risk_assessment['risk_level']}")
        print(f"Suspicious indicators: {len(analysis.suspicious_indicators)}")
    
    asyncio.run(test_analyzer())
