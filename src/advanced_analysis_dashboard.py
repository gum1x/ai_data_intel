#!/usr/bin/env python3
"""
Advanced Analysis Dashboard for Comprehensive Telegram Monitoring
Handles user profiling, suspicious activity detection, and AI learning analysis
"""

import sqlite3
import json
import pandas as pd
from datetime import datetime, timedelta
import matplotlib.pyplot as plt
import seaborn as sns
from collections import Counter, defaultdict
import re
from chatgptapibypass import chatgpt
import numpy as np

class AdvancedAnalysisDashboard:
    """Advanced dashboard for comprehensive Telegram analysis"""
    
    def __init__(self, db_file="advanced_telegram_monitor.db"):
        self.db_file = db_file
        self.conn = sqlite3.connect(db_file)
        
    def get_comprehensive_summary(self):
        """Get comprehensive system summary"""
        cursor = self.conn.cursor()
        
        summary = {}
        
        # Basic counts
        tables = ['user_profiles', 'raw_chat_data', 'suspicious_activities', 'ai_learning_data']
        for table in tables:
            cursor.execute(f"SELECT COUNT(*) FROM {table}")
            summary[f"{table}_count"] = cursor.fetchone()[0]
        
        # User profiling stats
        cursor.execute("SELECT COUNT(*) FROM user_profiles WHERE profile_analyzed_at > datetime('now', '-1 day')")
        summary['recently_profiled_users'] = cursor.fetchone()[0]
        
        # Suspicious activity stats
        cursor.execute("SELECT COUNT(*) FROM suspicious_activities WHERE created_at > datetime('now', '-1 day')")
        summary['recent_suspicious_activities'] = cursor.fetchone()[0]
        
        # AI learning stats
        cursor.execute("SELECT COUNT(*) FROM ai_learning_data WHERE created_at > datetime('now', '-1 day')")
        summary['recent_ai_learning_events'] = cursor.fetchone()[0]
        
        return summary
    
    def analyze_user_profiles(self):
        """Comprehensive user profile analysis"""
        cursor = self.conn.cursor()
        
        # Get all user profiles
        cursor.execute("""
            SELECT user_id, username, bio, username_analysis, bio_analysis, 
                   activity_analysis, ai_assessment, profile_analyzed_at
            FROM user_profiles
            ORDER BY profile_analyzed_at DESC
        """)
        
        profiles = cursor.fetchall()
        
        analysis = {
            'total_profiles': len(profiles),
            'high_value_users': [],
            'suspicious_users': [],
            'premium_users': [],
            'bot_accounts': [],
            'username_value_distribution': {},
            'bio_analysis_summary': {},
            'activity_patterns': {}
        }
        
        for profile in profiles:
            user_id, username, bio, username_analysis, bio_analysis, activity_analysis, ai_assessment, analyzed_at = profile
            
            try:
                # Parse JSON data
                username_data = json.loads(username_analysis) if username_analysis else {}
                bio_data = json.loads(bio_analysis) if bio_analysis else {}
                activity_data = json.loads(activity_analysis) if activity_analysis else {}
                ai_data = json.loads(ai_assessment) if ai_assessment else {}
                
                user_info = {
                    'user_id': user_id,
                    'username': username,
                    'bio': bio,
                    'username_analysis': username_data,
                    'bio_analysis': bio_data,
                    'activity_analysis': activity_data,
                    'ai_assessment': ai_data,
                    'analyzed_at': analyzed_at
                }
                
                # Categorize users
                if username_data.get('value_score', 0) >= 20:
                    analysis['high_value_users'].append(user_info)
                
                if bio_data.get('risk_level') == 'high' or bio_data.get('suspicious_score', 0) >= 15:
                    analysis['suspicious_users'].append(user_info)
                
                if username_data.get('patterns', []):
                    for pattern in username_data['patterns']:
                        if pattern not in analysis['username_value_distribution']:
                            analysis['username_value_distribution'][pattern] = 0
                        analysis['username_value_distribution'][pattern] += 1
                
                # Bio analysis summary
                if bio_data.get('business_indicators'):
                    for indicator in bio_data['business_indicators']:
                        if indicator not in analysis['bio_analysis_summary']:
                            analysis['bio_analysis_summary'][indicator] = 0
                        analysis['bio_analysis_summary'][indicator] += 1
                
                # Activity patterns
                activity_level = activity_data.get('activity_level', 'unknown')
                if activity_level not in analysis['activity_patterns']:
                    analysis['activity_patterns'][activity_level] = 0
                analysis['activity_patterns'][activity_level] += 1
                
            except Exception as e:
                print(f"Error analyzing profile {user_id}: {e}")
                continue
        
        return analysis
    
    def analyze_suspicious_activities(self):
        """Analyze suspicious activities and patterns"""
        cursor = self.conn.cursor()
        
        cursor.execute("""
            SELECT user_id, username, chat_title, message, indicators, timestamp
            FROM suspicious_activities
            ORDER BY created_at DESC
        """)
        
        activities = cursor.fetchall()
        
        analysis = {
            'total_activities': len(activities),
            'activity_types': {},
            'suspicious_users': {},
            'chat_analysis': {},
            'indicator_frequency': {},
            'recent_trends': {}
        }
        
        for activity in activities:
            user_id, username, chat_title, message, indicators, timestamp = activity
            
            try:
                indicators_data = json.loads(indicators) if indicators else []
                
                # Count activity types
                for indicator in indicators_data:
                    activity_type = indicator.split(':')[0] if ':' in indicator else indicator
                    if activity_type not in analysis['activity_types']:
                        analysis['activity_types'][activity_type] = 0
                    analysis['activity_types'][activity_type] += 1
                    
                    # Count specific indicators
                    if indicator not in analysis['indicator_frequency']:
                        analysis['indicator_frequency'][indicator] = 0
                    analysis['indicator_frequency'][indicator] += 1
                
                # Track suspicious users
                if username not in analysis['suspicious_users']:
                    analysis['suspicious_users'][username] = {
                        'user_id': user_id,
                        'activity_count': 0,
                        'activities': []
                    }
                
                analysis['suspicious_users'][username]['activity_count'] += 1
                analysis['suspicious_users'][username]['activities'].append({
                    'chat': chat_title,
                    'message': message,
                    'indicators': indicators_data,
                    'timestamp': timestamp
                })
                
                # Chat analysis
                if chat_title not in analysis['chat_analysis']:
                    analysis['chat_analysis'][chat_title] = 0
                analysis['chat_analysis'][chat_title] += 1
                
            except Exception as e:
                print(f"Error analyzing suspicious activity: {e}")
                continue
        
        return analysis
    
    def analyze_ai_learning_progress(self):
        """Analyze AI learning and adaptation progress"""
        cursor = self.conn.cursor()
        
        cursor.execute("""
            SELECT context, response, outcome, timestamp
            FROM ai_learning_data
            ORDER BY timestamp DESC
        """)
        
        learning_data = cursor.fetchall()
        
        analysis = {
            'total_interactions': len(learning_data),
            'success_rate': 0,
            'learning_trends': {},
            'context_effectiveness': {},
            'response_patterns': {},
            'recent_improvements': []
        }
        
        successful_interactions = 0
        context_success = defaultdict(list)
        
        for data in learning_data:
            context, response, outcome, timestamp = data
            
            try:
                context_data = json.loads(context) if context else {}
                outcome_data = json.loads(outcome) if outcome else {}
                
                if outcome_data.get('success', False):
                    successful_interactions += 1
                
                # Analyze context effectiveness
                context_key = self.extract_context_key(context_data)
                context_success[context_key].append(outcome_data.get('success', False))
                
                # Learning trends by time
                date = datetime.fromisoformat(timestamp).date()
                if date not in analysis['learning_trends']:
                    analysis['learning_trends'][date] = {'success': 0, 'total': 0}
                
                analysis['learning_trends'][date]['total'] += 1
                if outcome_data.get('success', False):
                    analysis['learning_trends'][date]['success'] += 1
                
            except Exception as e:
                print(f"Error analyzing learning data: {e}")
                continue
        
        # Calculate success rate
        if learning_data:
            analysis['success_rate'] = successful_interactions / len(learning_data)
        
        # Calculate context effectiveness
        for context_key, successes in context_success.items():
            if len(successes) >= 3:  # Only analyze contexts with enough data
                success_rate = sum(successes) / len(successes)
                analysis['context_effectiveness'][context_key] = {
                    'success_rate': success_rate,
                    'interaction_count': len(successes)
                }
        
        return analysis
    
    def extract_context_key(self, context_data):
        """Extract key features from context for analysis"""
        key_features = []
        
        if 'user_type' in context_data:
            key_features.append(f"user_type:{context_data['user_type']}")
        
        if 'group_type' in context_data:
            key_features.append(f"group_type:{context_data['group_type']}")
        
        if 'topic' in context_data:
            key_features.append(f"topic:{context_data['topic']}")
        
        return "|".join(key_features) if key_features else "unknown"
    
    def generate_ai_insights(self):
        """Generate AI-powered insights from all collected data"""
        try:
            # Get comprehensive data
            user_analysis = self.analyze_user_profiles()
            suspicious_analysis = self.analyze_suspicious_activities()
            learning_analysis = self.analyze_ai_learning_progress()
            
            prompt = f"""
            Analyze this comprehensive Telegram monitoring data and provide strategic insights:
            
            USER PROFILES ANALYSIS:
            {json.dumps(user_analysis, indent=2)}
            
            SUSPICIOUS ACTIVITIES ANALYSIS:
            {json.dumps(suspicious_analysis, indent=2)}
            
            AI LEARNING ANALYSIS:
            {json.dumps(learning_analysis, indent=2)}
            
            Provide insights on:
            1. High-value targets and opportunities
            2. Risk assessment and security concerns
            3. Optimal engagement strategies
            4. AI learning effectiveness and improvements
            5. Emerging patterns and trends
            6. Recommended actions and priorities
            7. Potential illegal activities and evidence
            8. User behavior predictions
            9. System optimization recommendations
            10. Strategic next steps
            
            Format as comprehensive analysis with actionable recommendations.
            """
            
            response = chatgpt(prompt)
            return response
            
        except Exception as e:
            return f"AI insights generation failed: {e}"
    
    def identify_high_value_targets(self):
        """Identify high-value targets for engagement"""
        cursor = self.conn.cursor()
        
        cursor.execute("""
            SELECT user_id, username, bio, username_analysis, bio_analysis, ai_assessment
            FROM user_profiles
            WHERE username_analysis IS NOT NULL AND bio_analysis IS NOT NULL
        """)
        
        profiles = cursor.fetchall()
        
        high_value_targets = []
        
        for profile in profiles:
            user_id, username, bio, username_analysis, bio_analysis, ai_assessment = profile
            
            try:
                username_data = json.loads(username_analysis)
                bio_data = json.loads(bio_analysis)
                ai_data = json.loads(ai_assessment) if ai_assessment else {}
                
                # Calculate value score
                value_score = 0
                
                # Username value
                value_score += username_data.get('value_score', 0)
                
                # Bio indicators
                if bio_data.get('business_indicators'):
                    value_score += len(bio_data['business_indicators']) * 5
                
                # AI assessment
                if ai_data.get('value_assessment'):
                    if 'high' in ai_data['value_assessment'].lower():
                        value_score += 20
                    elif 'medium' in ai_data['value_assessment'].lower():
                        value_score += 10
                
                # Contact information availability
                if bio_data.get('contact_info'):
                    value_score += len(bio_data['contact_info']) * 3
                
                if value_score >= 25:  # Threshold for high value
                    high_value_targets.append({
                        'user_id': user_id,
                        'username': username,
                        'bio': bio,
                        'value_score': value_score,
                        'username_analysis': username_data,
                        'bio_analysis': bio_data,
                        'ai_assessment': ai_data
                    })
                
            except Exception as e:
                print(f"Error evaluating target {user_id}: {e}")
                continue
        
        # Sort by value score
        high_value_targets.sort(key=lambda x: x['value_score'], reverse=True)
        
        return high_value_targets
    
    def detect_illegal_activity_patterns(self):
        """Detect patterns indicating illegal activities"""
        cursor = self.conn.cursor()
        
        cursor.execute("""
            SELECT user_id, username, chat_title, message, indicators, timestamp
            FROM suspicious_activities
            WHERE indicators LIKE '%illegal%' OR indicators LIKE '%scam%' OR indicators LIKE '%fraud%'
            ORDER BY timestamp DESC
        """)
        
        illegal_activities = cursor.fetchall()
        
        patterns = {
            'total_illegal_activities': len(illegal_activities),
            'user_patterns': {},
            'chat_patterns': {},
            'message_patterns': {},
            'timeline_analysis': {},
            'risk_assessment': {}
        }
        
        for activity in illegal_activities:
            user_id, username, chat_title, message, indicators, timestamp = activity
            
            try:
                indicators_data = json.loads(indicators) if indicators else []
                
                # User patterns
                if username not in patterns['user_patterns']:
                    patterns['user_patterns'][username] = {
                        'user_id': user_id,
                        'activity_count': 0,
                        'illegal_indicators': [],
                        'chats_involved': set()
                    }
                
                patterns['user_patterns'][username]['activity_count'] += 1
                patterns['user_patterns'][username]['illegal_indicators'].extend(indicators_data)
                patterns['user_patterns'][username]['chats_involved'].add(chat_title)
                
                # Chat patterns
                if chat_title not in patterns['chat_patterns']:
                    patterns['chat_patterns'][chat_title] = 0
                patterns['chat_patterns'][chat_title] += 1
                
                # Message patterns
                if message:
                    # Extract keywords
                    keywords = re.findall(r'\b\w+\b', message.lower())
                    for keyword in keywords:
                        if keyword not in patterns['message_patterns']:
                            patterns['message_patterns'][keyword] = 0
                        patterns['message_patterns'][keyword] += 1
                
                # Timeline analysis
                date = datetime.fromisoformat(timestamp).date()
                if date not in patterns['timeline_analysis']:
                    patterns['timeline_analysis'][date] = 0
                patterns['timeline_analysis'][date] += 1
                
            except Exception as e:
                print(f"Error analyzing illegal activity: {e}")
                continue
        
        # Convert sets to lists for JSON serialization
        for user_data in patterns['user_patterns'].values():
            user_data['chats_involved'] = list(user_data['chats_involved'])
        
        # Risk assessment
        for username, user_data in patterns['user_patterns'].items():
            risk_score = 0
            risk_score += user_data['activity_count'] * 5
            risk_score += len(user_data['chats_involved']) * 3
            risk_score += len(user_data['illegal_indicators']) * 2
            
            if risk_score >= 20:
                risk_level = 'high'
            elif risk_score >= 10:
                risk_level = 'medium'
            else:
                risk_level = 'low'
            
            patterns['risk_assessment'][username] = {
                'risk_score': risk_score,
                'risk_level': risk_level,
                'user_data': user_data
            }
        
        return patterns
    
    def create_comprehensive_report(self):
        """Create comprehensive analysis report"""
        report = {
            'generated_at': datetime.now().isoformat(),
            'summary': self.get_comprehensive_summary(),
            'user_analysis': self.analyze_user_profiles(),
            'suspicious_activities': self.analyze_suspicious_activities(),
            'ai_learning': self.analyze_ai_learning_progress(),
            'high_value_targets': self.identify_high_value_targets(),
            'illegal_activity_patterns': self.detect_illegal_activity_patterns(),
            'ai_insights': self.generate_ai_insights()
        }
        
        return report
    
    def export_comprehensive_report(self, filename="comprehensive_analysis_report.json"):
        """Export comprehensive report to file"""
        report = self.create_comprehensive_report()
        
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        print(f"Comprehensive report exported to: {filename}")
        return report
    
    def interactive_advanced_analysis(self):
        """Interactive advanced analysis mode"""
        while True:
            print("\n=== ADVANCED TELEGRAM ANALYSIS DASHBOARD ===")
            print("1. Comprehensive Summary")
            print("2. User Profile Analysis")
            print("3. Suspicious Activities Analysis")
            print("4. AI Learning Progress")
            print("5. High-Value Targets")
            print("6. Illegal Activity Detection")
            print("7. Generate AI Insights")
            print("8. Export Comprehensive Report")
            print("9. Create Advanced Visualizations")
            print("0. Exit")
            
            choice = input("\nSelect option (0-9): ").strip()
            
            if choice == '0':
                break
            elif choice == '1':
                summary = self.get_comprehensive_summary()
                print("\nComprehensive Summary:")
                for key, value in summary.items():
                    print(f"  {key}: {value}")
            elif choice == '2':
                analysis = self.analyze_user_profiles()
                print(f"\nUser Profile Analysis:")
                print(f"  Total profiles: {analysis['total_profiles']}")
                print(f"  High-value users: {len(analysis['high_value_users'])}")
                print(f"  Suspicious users: {len(analysis['suspicious_users'])}")
                print(f"  Username patterns: {analysis['username_value_distribution']}")
            elif choice == '3':
                analysis = self.analyze_suspicious_activities()
                print(f"\nSuspicious Activities Analysis:")
                print(f"  Total activities: {analysis['total_activities']}")
                print(f"  Activity types: {analysis['activity_types']}")
                print(f"  Most suspicious users: {list(analysis['suspicious_users'].keys())[:5]}")
            elif choice == '4':
                analysis = self.analyze_ai_learning_progress()
                print(f"\nAI Learning Progress:")
                print(f"  Total interactions: {analysis['total_interactions']}")
                print(f"  Success rate: {analysis['success_rate']:.2%}")
                print(f"  Context effectiveness: {analysis['context_effectiveness']}")
            elif choice == '5':
                targets = self.identify_high_value_targets()
                print(f"\nHigh-Value Targets ({len(targets)} found):")
                for i, target in enumerate(targets[:10], 1):
                    print(f"  {i}. {target['username']} (Score: {target['value_score']})")
            elif choice == '6':
                patterns = self.detect_illegal_activity_patterns()
                print(f"\nIllegal Activity Patterns:")
                print(f"  Total illegal activities: {patterns['total_illegal_activities']}")
                print(f"  High-risk users: {len([u for u in patterns['risk_assessment'].values() if u['risk_level'] == 'high'])}")
                print(f"  Most active illegal chats: {list(patterns['chat_patterns'].keys())[:5]}")
            elif choice == '7':
                print("\nGenerating AI insights...")
                insights = self.generate_ai_insights()
                print(insights)
            elif choice == '8':
                self.export_comprehensive_report()
            elif choice == '9':
                self.create_advanced_visualizations()
            else:
                print("Invalid option. Please try again.")
    
    def create_advanced_visualizations(self):
        """Create advanced visualizations"""
        try:
            fig, axes = plt.subplots(3, 2, figsize=(16, 18))
            fig.suptitle('Advanced Telegram Analysis Dashboard', fontsize=16)
            
            # 1. User value distribution
            user_analysis = self.analyze_user_profiles()
            username_patterns = user_analysis['username_value_distribution']
            if username_patterns:
                axes[0, 0].pie(username_patterns.values(), labels=username_patterns.keys(), autopct='%1.1f%%')
                axes[0, 0].set_title('Username Value Patterns')
            
            # 2. Suspicious activities by type
            suspicious_analysis = self.analyze_suspicious_activities()
            activity_types = suspicious_analysis['activity_types']
            if activity_types:
                axes[0, 1].bar(activity_types.keys(), activity_types.values())
                axes[0, 1].set_title('Suspicious Activities by Type')
                axes[0, 1].tick_params(axis='x', rotation=45)
            
            # 3. AI learning success rate over time
            learning_analysis = self.analyze_ai_learning_progress()
            trends = learning_analysis['learning_trends']
            if trends:
                dates = list(trends.keys())
                success_rates = [trends[date]['success'] / trends[date]['total'] if trends[date]['total'] > 0 else 0 for date in dates]
                axes[1, 0].plot(dates, success_rates, marker='o')
                axes[1, 0].set_title('AI Learning Success Rate Over Time')
                axes[1, 0].set_ylabel('Success Rate')
                axes[1, 0].tick_params(axis='x', rotation=45)
            
            # 4. High-value targets
            targets = self.identify_high_value_targets()
            if targets:
                usernames = [t['username'][:15] + '...' if len(t['username']) > 15 else t['username'] for t in targets[:10]]
                scores = [t['value_score'] for t in targets[:10]]
                axes[1, 1].bar(range(len(usernames)), scores)
                axes[1, 1].set_title('Top High-Value Targets')
                axes[1, 1].set_xticks(range(len(usernames)))
                axes[1, 1].set_xticklabels(usernames, rotation=45, ha='right')
                axes[1, 1].set_ylabel('Value Score')
            
            # 5. Illegal activity timeline
            illegal_patterns = self.detect_illegal_activity_patterns()
            timeline = illegal_patterns['timeline_analysis']
            if timeline:
                dates = list(timeline.keys())
                counts = list(timeline.values())
                axes[2, 0].plot(dates, counts, marker='o', color='red')
                axes[2, 0].set_title('Illegal Activities Timeline')
                axes[2, 0].set_ylabel('Activity Count')
                axes[2, 0].tick_params(axis='x', rotation=45)
            
            # 6. Risk assessment distribution
            risk_assessment = illegal_patterns['risk_assessment']
            if risk_assessment:
                risk_levels = [data['risk_level'] for data in risk_assessment.values()]
                risk_counts = Counter(risk_levels)
                axes[2, 1].pie(risk_counts.values(), labels=risk_counts.keys(), autopct='%1.1f%%')
                axes[2, 1].set_title('User Risk Level Distribution')
            
            plt.tight_layout()
            plt.savefig('advanced_analysis_dashboard.png', dpi=300, bbox_inches='tight')
            plt.show()
            
            print("Advanced visualizations saved as 'advanced_analysis_dashboard.png'")
            
        except Exception as e:
            print(f"Visualization error: {e}")
    
    def close(self):
        """Close database connection"""
        self.conn.close()

def main():
    """Main function"""
    dashboard = AdvancedAnalysisDashboard()
    
    try:
        dashboard.interactive_advanced_analysis()
    finally:
        dashboard.close()

if __name__ == "__main__":
    main()
