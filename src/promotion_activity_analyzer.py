#!/usr/bin/env python3
"""
Promotion Activity Analyzer
Advanced analysis of giveaways, promotions, and social engineering tactics
"""

import asyncio
import json
import re
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Set
from dataclasses import dataclass
from enum import Enum
import logging
from collections import defaultdict

class PromotionType(Enum):
    GIVEAWAY = "giveaway"
    AIRDROP = "airdrop"
    CONTEST = "contest"
    RAFFLE = "raffle"
    WHITELIST = "whitelist"
    FREE_MINT = "free_mint"
    PRESALE = "presale"
    DISCOUNT = "discount"

@dataclass
class PromotionDetails:
    """Details of a promotional activity"""
    promotion_id: str
    promotion_type: PromotionType
    creator_id: str
    creator_username: str
    start_time: datetime
    end_time: Optional[datetime]
    requirements: List[str]
    prize_description: str
    participant_count: int
    channel_ids: List[str]
    is_suspicious: bool
    risk_score: float
    analysis: Dict[str, Any]

class PromotionActivityAnalyzer:
    """Analyzer for promotional activities and giveaways"""
    
    def __init__(self):
        self.known_scam_patterns = self.load_scam_patterns()
        self.suspicious_prize_keywords = self.load_suspicious_keywords()
        self.promotion_history = {}
        self.user_participation_patterns = defaultdict(list)
        self.channel_promotion_patterns = defaultdict(list)
    
    def load_scam_patterns(self) -> Dict[str, List[str]]:
        """Load known scam patterns"""
        return {
            'urgency_patterns': [
                r'only \d+ spots left',
                r'ending soon',
                r'last chance',
                r'hurry up',
                r'don\'t miss out',
                r'limited time',
                r'\d+ minutes left'
            ],
            'fake_prize_patterns': [
                r'\d+ eth',
                r'\d+ bitcoin',
                r'free nft',
                r'whitelist spot',
                r'guaranteed allocation',
                r'exclusive access',
                r'og role'
            ],
            'requirement_patterns': [
                r'send \w+',
                r'deposit required',
                r'verification fee',
                r'gas fee required',
                r'minimum balance',
                r'connect wallet',
                r'validate wallet'
            ],
            'manipulation_patterns': [
                r'100% legit',
                r'not a scam',
                r'trust me',
                r'no scam',
                r'real giveaway',
                r'official giveaway',
                r'admin here'
            ]
        }
    
    def load_suspicious_keywords(self) -> Set[str]:
        """Load suspicious prize keywords"""
        return {
            'free eth', 'free bitcoin', 'free bnb', 'free sol',
            'whitelist spot', 'presale access', 'og role',
            'exclusive nft', 'rare nft', 'legendary nft',
            'guaranteed allocation', 'free mint', 'stealth mint',
            'verified profit', 'easy money', 'quick profit'
        }
    
    async def analyze_promotion(self, message_data: Dict[str, Any]) -> Optional[PromotionDetails]:
        """Analyze a potential promotional message"""
        try:
            # Extract basic info
            text = message_data.get('text', '').lower()
            user_id = message_data.get('user_id', '')
            username = message_data.get('username', '')
            
            # Detect promotion type
            promotion_type = self.detect_promotion_type(text)
            if not promotion_type:
                return None
            
            # Extract promotion details
            requirements = self.extract_requirements(text)
            prize_description = self.extract_prize_description(text)
            time_info = self.extract_time_information(text)
            
            # Analyze risk
            risk_analysis = self.analyze_promotion_risk(
                text, requirements, prize_description, message_data
            )
            
            # Create promotion details
            promotion = PromotionDetails(
                promotion_id=f"promo_{hash(text)}_{int(datetime.now().timestamp())}",
                promotion_type=promotion_type,
                creator_id=user_id,
                creator_username=username,
                start_time=datetime.now(),
                end_time=time_info.get('end_time'),
                requirements=requirements,
                prize_description=prize_description,
                participant_count=0,  # Will be updated as users participate
                channel_ids=[message_data.get('chat_id', '')],
                is_suspicious=risk_analysis['is_suspicious'],
                risk_score=risk_analysis['risk_score'],
                analysis=risk_analysis
            )
            
            # Store in history
            self.promotion_history[promotion.promotion_id] = promotion
            
            # Update channel patterns
            self.update_channel_patterns(promotion)
            
            return promotion
            
        except Exception as e:
            logging.error(f"Promotion analysis error: {e}")
            return None
    
    def detect_promotion_type(self, text: str) -> Optional[PromotionType]:
        """Detect type of promotion"""
        text = text.lower()
        
        if 'giveaway' in text:
            return PromotionType.GIVEAWAY
        elif 'airdrop' in text:
            return PromotionType.AIRDROP
        elif 'contest' in text:
            return PromotionType.CONTEST
        elif 'raffle' in text:
            return PromotionType.RAFFLE
        elif 'whitelist' in text:
            return PromotionType.WHITELIST
        elif 'free mint' in text:
            return PromotionType.FREE_MINT
        elif 'presale' in text:
            return PromotionType.PRESALE
        elif any(word in text for word in ['discount', 'off', 'sale']):
            return PromotionType.DISCOUNT
        
        return None
    
    def extract_requirements(self, text: str) -> List[str]:
        """Extract participation requirements"""
        requirements = []
        
        # Common requirement patterns
        requirement_patterns = [
            r'must \w+',
            r'need to \w+',
            r'follow \w+',
            r'join \w+',
            r'retweet \w+',
            r'tag \w+',
            r'invite \w+',
            r'like \w+',
            r'share \w+',
            r'subscribe \w+',
            r'verify \w+',
            r'connect \w+',
            r'deposit \w+',
            r'send \w+'
        ]
        
        for pattern in requirement_patterns:
            matches = re.finditer(pattern, text.lower())
            for match in matches:
                requirement = text[match.start():].split('\n')[0].strip()
                if requirement:
                    requirements.append(requirement)
        
        return list(set(requirements))  # Remove duplicates
    
    def extract_prize_description(self, text: str) -> str:
        """Extract prize description"""
        prize = ""
        
        # Prize patterns
        prize_patterns = [
            r'\d+\s*(?:eth|btc|bnb|sol|usd)',
            r'\d+\s*(?:nft|whitelist spots|tokens)',
            r'(?:legendary|rare|exclusive|special)\s*\w+',
            r'og role',
            r'whitelist spot',
            r'exclusive access',
            r'guaranteed allocation'
        ]
        
        for pattern in prize_patterns:
            matches = re.finditer(pattern, text.lower())
            for match in matches:
                prize_part = text[match.start():].split('\n')[0].strip()
                if prize_part:
                    prize += prize_part + "; "
        
        return prize.strip("; ")
    
    def extract_time_information(self, text: str) -> Dict[str, Optional[datetime]]:
        """Extract time-related information"""
        time_info = {
            'start_time': None,
            'end_time': None
        }
        
        # Time patterns
        time_patterns = {
            'ends_in_hours': r'ends in (\d+)\s*hours?',
            'ends_in_minutes': r'ends in (\d+)\s*minutes?',
            'ends_tomorrow': r'ends tomorrow',
            'hours_left': r'(\d+)\s*hours? left',
            'minutes_left': r'(\d+)\s*minutes? left'
        }
        
        now = datetime.now()
        
        for pattern_name, pattern in time_patterns.items():
            match = re.search(pattern, text.lower())
            if match:
                if 'hours' in pattern_name and match.group(1):
                    hours = int(match.group(1))
                    time_info['end_time'] = now + timedelta(hours=hours)
                elif 'minutes' in pattern_name and match.group(1):
                    minutes = int(match.group(1))
                    time_info['end_time'] = now + timedelta(minutes=minutes)
                elif pattern_name == 'ends_tomorrow':
                    time_info['end_time'] = now + timedelta(days=1)
        
        return time_info
    
    def analyze_promotion_risk(self, text: str, requirements: List[str], 
                             prize_description: str, message_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze promotion for risk factors"""
        risk_analysis = {
            'is_suspicious': False,
            'risk_score': 0.0,
            'risk_factors': [],
            'scam_indicators': [],
            'manipulation_tactics': [],
            'urgency_indicators': []
        }
        
        # Check for scam patterns
        for category, patterns in self.known_scam_patterns.items():
            for pattern in patterns:
                if re.search(pattern, text.lower()):
                    risk_analysis['risk_score'] += 0.2
                    risk_analysis['scam_indicators'].append(f"{category}: {pattern}")
        
        # Check requirements
        suspicious_requirements = [
            req for req in requirements
            if any(word in req.lower() for word in ['send', 'deposit', 'fee', 'connect', 'validate'])
        ]
        if suspicious_requirements:
            risk_analysis['risk_score'] += 0.3
            risk_analysis['risk_factors'].extend(suspicious_requirements)
        
        # Check prize description
        if any(keyword in prize_description.lower() for keyword in self.suspicious_prize_keywords):
            risk_analysis['risk_score'] += 0.2
            risk_analysis['risk_factors'].append('suspicious_prize')
        
        # Check for urgency tactics
        urgency_words = ['quick', 'fast', 'hurry', 'limited', 'last chance', 'ending']
        if any(word in text.lower() for word in urgency_words):
            risk_analysis['risk_score'] += 0.1
            risk_analysis['urgency_indicators'].extend(
                [word for word in urgency_words if word in text.lower()]
            )
        
        # Check creator history
        creator_history = self.get_creator_history(message_data.get('user_id', ''))
        if creator_history.get('previous_scams', 0) > 0:
            risk_analysis['risk_score'] += 0.4
            risk_analysis['risk_factors'].append('creator_history')
        
        # Check for manipulation tactics
        manipulation_words = ['trust', 'legit', 'real', 'official', 'verified', 'guaranteed']
        if any(word in text.lower() for word in manipulation_words):
            risk_analysis['risk_score'] += 0.1
            risk_analysis['manipulation_tactics'].extend(
                [word for word in manipulation_words if word in text.lower()]
            )
        
        # Final risk assessment
        risk_analysis['risk_score'] = min(risk_analysis['risk_score'], 1.0)
        risk_analysis['is_suspicious'] = risk_analysis['risk_score'] > 0.5
        
        return risk_analysis
    
    def get_creator_history(self, creator_id: str) -> Dict[str, Any]:
        """Get creator's promotion history"""
        history = {
            'total_promotions': 0,
            'suspicious_promotions': 0,
            'previous_scams': 0,
            'average_risk_score': 0.0
        }
        
        creator_promotions = [
            promo for promo in self.promotion_history.values()
            if promo.creator_id == creator_id
        ]
        
        if creator_promotions:
            history['total_promotions'] = len(creator_promotions)
            history['suspicious_promotions'] = len(
                [p for p in creator_promotions if p.is_suspicious]
            )
            history['previous_scams'] = len(
                [p for p in creator_promotions if p.risk_score > 0.8]
            )
            history['average_risk_score'] = sum(
                p.risk_score for p in creator_promotions
            ) / len(creator_promotions)
        
        return history
    
    def update_channel_patterns(self, promotion: PromotionDetails):
        """Update channel promotion patterns"""
        for channel_id in promotion.channel_ids:
            self.channel_promotion_patterns[channel_id].append({
                'promotion_id': promotion.promotion_id,
                'promotion_type': promotion.promotion_type,
                'timestamp': datetime.now(),
                'risk_score': promotion.risk_score
            })
            
            # Keep only recent history
            self.channel_promotion_patterns[channel_id] = [
                p for p in self.channel_promotion_patterns[channel_id]
                if p['timestamp'] > datetime.now() - timedelta(days=30)
            ]
    
    def analyze_channel_promotion_patterns(self, channel_id: str) -> Dict[str, Any]:
        """Analyze promotion patterns in a channel"""
        patterns = self.channel_promotion_patterns.get(channel_id, [])
        
        if not patterns:
            return {
                'total_promotions': 0,
                'risk_level': 'unknown',
                'suspicious_ratio': 0.0,
                'common_types': []
            }
        
        analysis = {
            'total_promotions': len(patterns),
            'suspicious_count': len([p for p in patterns if p['risk_score'] > 0.5]),
            'high_risk_count': len([p for p in patterns if p['risk_score'] > 0.8]),
            'promotion_types': Counter([p['promotion_type'].value for p in patterns]),
            'average_risk_score': sum(p['risk_score'] for p in patterns) / len(patterns),
            'suspicious_ratio': 0.0,
            'risk_level': 'low',
            'common_types': [],
            'temporal_patterns': self.analyze_temporal_patterns(patterns)
        }
        
        # Calculate suspicious ratio
        analysis['suspicious_ratio'] = analysis['suspicious_count'] / analysis['total_promotions']
        
        # Determine risk level
        if analysis['average_risk_score'] > 0.8:
            analysis['risk_level'] = 'critical'
        elif analysis['average_risk_score'] > 0.6:
            analysis['risk_level'] = 'high'
        elif analysis['average_risk_score'] > 0.4:
            analysis['risk_level'] = 'medium'
        
        # Get common promotion types
        analysis['common_types'] = [
            type_name for type_name, count in analysis['promotion_types'].most_common(3)
        ]
        
        return analysis
    
    def analyze_temporal_patterns(self, patterns: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze temporal patterns of promotions"""
        timestamps = [p['timestamp'] for p in patterns]
        
        if not timestamps:
            return {}
        
        # Sort timestamps
        timestamps.sort()
        
        # Calculate intervals
        intervals = []
        for i in range(1, len(timestamps)):
            interval = (timestamps[i] - timestamps[i-1]).total_seconds() / 3600  # hours
            intervals.append(interval)
        
        if not intervals:
            return {}
        
        return {
            'average_interval': sum(intervals) / len(intervals),
            'min_interval': min(intervals),
            'max_interval': max(intervals),
            'burst_patterns': self.detect_burst_patterns(intervals),
            'time_of_day_distribution': self.analyze_time_distribution(timestamps)
        }
    
    def detect_burst_patterns(self, intervals: List[float]) -> List[Dict[str, Any]]:
        """Detect burst patterns in promotion activity"""
        bursts = []
        current_burst = []
        
        for i, interval in enumerate(intervals):
            if interval < 1.0:  # Less than 1 hour
                current_burst.append(interval)
            else:
                if len(current_burst) > 2:  # At least 3 promotions in burst
                    bursts.append({
                        'size': len(current_burst) + 1,
                        'average_interval': sum(current_burst) / len(current_burst),
                        'position': i - len(current_burst)
                    })
                current_burst = []
        
        # Check last burst
        if len(current_burst) > 2:
            bursts.append({
                'size': len(current_burst) + 1,
                'average_interval': sum(current_burst) / len(current_burst),
                'position': len(intervals) - len(current_burst)
            })
        
        return bursts
    
    def analyze_time_distribution(self, timestamps: List[datetime]) -> Dict[int, int]:
        """Analyze distribution of promotion times"""
        hour_distribution = defaultdict(int)
        
        for ts in timestamps:
            hour_distribution[ts.hour] += 1
        
        return dict(hour_distribution)
    
    def get_promotion_recommendations(self, promotion: PromotionDetails) -> List[str]:
        """Get recommendations based on promotion analysis"""
        recommendations = []
        
        if promotion.is_suspicious:
            recommendations.append("HIGH RISK: Monitor this promotion closely")
            
            if promotion.risk_score > 0.8:
                recommendations.append("CRITICAL: Consider immediate intervention")
            
            if any('deposit' in req.lower() for req in promotion.requirements):
                recommendations.append("WARNING: Requires deposit/payment - Likely scam")
            
            if any('connect' in req.lower() for req in promotion.requirements):
                recommendations.append("CAUTION: Requests wallet connection - Potential security risk")
        
        if promotion.analysis.get('urgency_indicators'):
            recommendations.append("NOTE: Uses urgency tactics - May be manipulative")
        
        if promotion.analysis.get('manipulation_tactics'):
            recommendations.append("ALERT: Uses manipulation tactics - Exercise caution")
        
        # Add channel-specific recommendations
        for channel_id in promotion.channel_ids:
            channel_analysis = self.analyze_channel_promotion_patterns(channel_id)
            if channel_analysis['risk_level'] in ['high', 'critical']:
                recommendations.append(f"WARNING: Channel has history of suspicious promotions")
        
        return recommendations

# Example usage
async def main():
    """Example usage of PromotionActivityAnalyzer"""
    analyzer = PromotionActivityAnalyzer()
    
    # Example promotion message
    message_data = {
        'text': """🎉 HUGE GIVEAWAY! 🎉
        Win 5 ETH + Rare NFT! 🚀
        
        Requirements:
        1. Follow @crypto_trader
        2. Join our channel
        3. Tag 3 friends
        4. Connect wallet to verify
        
        Only 100 spots left! Hurry up! ⚡
        Ends in 24 hours!
        
        100% LEGIT - Admin Verified ✅""",
        'user_id': '123456789',
        'username': 'crypto_trader',
        'chat_id': 'channel_123'
    }
    
    # Analyze promotion
    promotion = await analyzer.analyze_promotion(message_data)
    
    if promotion:
        print(f"Promotion Type: {promotion.promotion_type.value}")
        print(f"Risk Score: {promotion.risk_score}")
        print(f"Is Suspicious: {promotion.is_suspicious}")
        print("\nRequirements:")
        for req in promotion.requirements:
            print(f"- {req}")
        print("\nRisk Analysis:")
        for key, value in promotion.analysis.items():
            print(f"{key}: {value}")
        print("\nRecommendations:")
        for rec in analyzer.get_promotion_recommendations(promotion):
            print(f"- {rec}")

if __name__ == "__main__":
    asyncio.run(main())
