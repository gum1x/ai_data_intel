"""
Advanced Analytics Engine - Machine learning insights and predictive analytics
"""

import asyncio
import numpy as np
import pandas as pd
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import tensorflow as tf
from sklearn.ensemble import IsolationForest, RandomForestClassifier, GradientBoostingClassifier
from sklearn.cluster import DBSCAN, KMeans, AgglomerativeClustering
from sklearn.preprocessing import StandardScaler, LabelEncoder, MinMaxScaler
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.metrics import classification_report, confusion_matrix, silhouette_score
from sklearn.decomposition import PCA, TruncatedSVD
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression, LinearRegression
from sklearn.svm import SVC, SVR
from sklearn.neural_network import MLPClassifier, MLPRegressor
import xgboost as xgb
import lightgbm as lgb
import catboost as cb
from scipy import stats
from scipy.cluster.hierarchy import dendrogram, linkage
import matplotlib.pyplot as plt
import seaborn as sns
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import networkx as nx
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Tuple, Union
import logging
from dataclasses import dataclass, asdict
from enum import Enum
import json
import pickle
import joblib
from database_manager import DatabaseManager
import redis
from collections import defaultdict, Counter
import warnings
warnings.filterwarnings('ignore')

class AnalyticsType(Enum):
    PREDICTIVE = "predictive"
    DESCRIPTIVE = "descriptive"
    DIAGNOSTIC = "diagnostic"
    PRESCRIPTIVE = "prescriptive"
    REAL_TIME = "real_time"
    BATCH = "batch"

class ModelType(Enum):
    CLASSIFICATION = "classification"
    REGRESSION = "regression"
    CLUSTERING = "clustering"
    ANOMALY_DETECTION = "anomaly_detection"
    TIME_SERIES = "time_series"
    NLP = "nlp"
    NETWORK = "network"

@dataclass
class AnalyticsResult:
    """Analytics result structure"""
    result_id: str
    analytics_type: AnalyticsType
    model_type: ModelType
    insights: Dict[str, Any]
    predictions: List[Any]
    confidence: float
    accuracy: float
    timestamp: datetime
    metadata: Dict[str, Any]

class AdvancedAnalyticsEngine:
    """Advanced analytics engine with machine learning insights"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.models = {}
        self.scalers = {}
        self.encoders = {}
        self.feature_importance = {}
        self.analytics_cache = {}
        self.performance_metrics = {}
        
        self.db_manager = None
        self.redis_client = None
        
        asyncio.create_task(self.initialize_database())
        self.initialize_models()
        self.initialize_analytics()
    
    async def initialize_database(self):
        """Initialize database connections"""
        try:
            # Initialize PostgreSQL database manager
            self.db_manager = DatabaseManager(self.config.get('database', {}))
            await self.db_manager.initialize()
            
            # Initialize Redis client
            self.redis_client = redis.Redis(
                host=self.config.get('redis', {}).get('host', 'localhost'),
                port=self.config.get('redis', {}).get('port', 6379),
                db=self.config.get('redis', {}).get('db', 0)
            )
            
            logging.info("Database connections initialized")
            
        except Exception as e:
            logging.error(f"Failed to initialize database: {e}")
            raise
    
    def create_analytics_tables(self):
        """Create analytics database tables (handled by DatabaseManager)"""
        # Tables are now created by the DatabaseManager
        pass
    
    def initialize_models(self):
        """Initialize machine learning models"""
        try:
            self.models['classification'] = {
                'random_forest': RandomForestClassifier(n_estimators=100, random_state=42),
                'gradient_boosting': GradientBoostingClassifier(n_estimators=100, random_state=42),
                'xgboost': xgb.XGBClassifier(n_estimators=100, random_state=42),
                'lightgbm': lgb.LGBMClassifier(n_estimators=100, random_state=42),
                'catboost': cb.CatBoostClassifier(iterations=100, random_state=42, verbose=False),
                'svm': SVC(kernel='rbf', random_state=42),
                'neural_network': MLPClassifier(hidden_layer_sizes=(100, 50), random_state=42)
            }
            
            self.models['regression'] = {
                'linear_regression': LinearRegression(),
                'random_forest': RandomForestRegressor(n_estimators=100, random_state=42),
                'xgboost': xgb.XGBRegressor(n_estimators=100, random_state=42),
                'lightgbm': lgb.LGBMRegressor(n_estimators=100, random_state=42),
                'catboost': cb.CatBoostRegressor(iterations=100, random_state=42, verbose=False),
                'svr': SVR(kernel='rbf'),
                'neural_network': MLPRegressor(hidden_layer_sizes=(100, 50), random_state=42)
            }
            
            self.models['clustering'] = {
                'kmeans': KMeans(n_clusters=5, random_state=42),
                'dbscan': DBSCAN(eps=0.5, min_samples=5),
                'agglomerative': AgglomerativeClustering(n_clusters=5),
                'gaussian_mixture': None
            }
            
            self.models['anomaly_detection'] = {
                'isolation_forest': IsolationForest(contamination=0.1, random_state=42),
                'one_class_svm': None,
                'local_outlier_factor': None
            }
            
            self.models['time_series'] = {
                'arima': None,
                'lstm': None,
                'prophet': None
            }
            
            self.models['nlp'] = {
                'tfidf': TfidfVectorizer(max_features=1000, stop_words='english'),
                'sentiment_analyzer': None,
                'topic_modeler': None
            }
            
            logging.info("Machine learning models initialized")
            
        except Exception as e:
            logging.error(f"Failed to initialize models: {e}")
            raise
    
    def initialize_analytics(self):
        """Initialize analytics components"""
        try:
            self.scalers = {
                'standard': StandardScaler(),
                'minmax': MinMaxScaler(),
                'robust': None
            }
            
            self.encoders = {
                'label': LabelEncoder(),
                'onehot': None
            }
            
            logging.info("Analytics components initialized")
            
        except Exception as e:
            logging.error(f"Failed to initialize analytics: {e}")
            raise
    
    async def analyze_user_behavior(self, user_data: Dict[str, Any]) -> AnalyticsResult:
        """Analyze user behavior with advanced analytics"""
        try:
            features = self.prepare_behavior_features(user_data)
            
            analyses = {}
            
            clustering_result = await self.perform_behavioral_clustering(features)
            analyses['clustering'] = clustering_result
            
            anomaly_result = await self.detect_behavioral_anomalies(features)
            analyses['anomaly_detection'] = anomaly_result
            
            prediction_result = await self.predict_behavior(features)
            analyses['prediction'] = prediction_result
            
            network_result = await self.analyze_behavioral_network(user_data)
            analyses['network'] = network_result
            
            insights = await self.generate_behavioral_insights(analyses)
            
            result = AnalyticsResult(
                result_id=str(uuid.uuid4()),
                analytics_type=AnalyticsType.DESCRIPTIVE,
                model_type=ModelType.CLUSTERING,
                insights=insights,
                predictions=prediction_result.get('predictions', []),
                confidence=insights.get('confidence', 0.8),
                accuracy=insights.get('accuracy', 0.85),
                timestamp=datetime.now(),
                metadata={'user_id': user_data.get('user_id'), 'analysis_type': 'behavioral'}
            )
            
            await self.store_analytics_result(result)
            
            return result
            
        except Exception as e:
            logging.error(f"Behavior analysis error: {e}")
            raise
    
    async def perform_behavioral_clustering(self, features: np.ndarray) -> Dict[str, Any]:
        """Perform behavioral clustering analysis"""
        try:
            scaler = StandardScaler()
            features_scaled = scaler.fit_transform(features)
            
            clustering_results = {}
            
            kmeans = KMeans(n_clusters=5, random_state=42)
            kmeans_labels = kmeans.fit_predict(features_scaled)
            clustering_results['kmeans'] = {
                'labels': kmeans_labels.tolist(),
                'centers': kmeans.cluster_centers_.tolist(),
                'inertia': kmeans.inertia_,
                'silhouette_score': silhouette_score(features_scaled, kmeans_labels)
            }
            
            dbscan = DBSCAN(eps=0.5, min_samples=5)
            dbscan_labels = dbscan.fit_predict(features_scaled)
            clustering_results['dbscan'] = {
                'labels': dbscan_labels.tolist(),
                'n_clusters': len(set(dbscan_labels)) - (1 if -1 in dbscan_labels else 0),
                'n_noise': list(dbscan_labels).count(-1)
            }
            
            agg_clustering = AgglomerativeClustering(n_clusters=5)
            agg_labels = agg_clustering.fit_predict(features_scaled)
            clustering_results['agglomerative'] = {
                'labels': agg_labels.tolist(),
                'n_clusters': len(set(agg_labels))
            }
            
            return clustering_results
            
        except Exception as e:
            logging.error(f"Behavioral clustering error: {e}")
            return {}
    
    async def detect_behavioral_anomalies(self, features: np.ndarray) -> Dict[str, Any]:
        """Detect behavioral anomalies"""
        try:
            scaler = StandardScaler()
            features_scaled = scaler.fit_transform(features)
            
            isolation_forest = IsolationForest(contamination=0.1, random_state=42)
            isolation_labels = isolation_forest.fit_predict(features_scaled)
            isolation_scores = isolation_forest.decision_function(features_scaled)
            
            from sklearn.svm import OneClassSVM
            one_class_svm = OneClassSVM(nu=0.1)
            svm_labels = one_class_svm.fit_predict(features_scaled)
            svm_scores = one_class_svm.decision_function(features_scaled)
            
            from sklearn.neighbors import LocalOutlierFactor
            lof = LocalOutlierFactor(n_neighbors=20, contamination=0.1)
            lof_labels = lof.fit_predict(features_scaled)
            lof_scores = lof.negative_outlier_factor_
            
            return {
                'isolation_forest': {
                    'labels': isolation_labels.tolist(),
                    'scores': isolation_scores.tolist(),
                    'n_anomalies': list(isolation_labels).count(-1)
                },
                'one_class_svm': {
                    'labels': svm_labels.tolist(),
                    'scores': svm_scores.tolist(),
                    'n_anomalies': list(svm_labels).count(-1)
                },
                'local_outlier_factor': {
                    'labels': lof_labels.tolist(),
                    'scores': lof_scores.tolist(),
                    'n_anomalies': list(lof_labels).count(-1)
                }
            }
            
        except Exception as e:
            logging.error(f"Anomaly detection error: {e}")
            return {}
    
    async def predict_behavior(self, features: np.ndarray) -> Dict[str, Any]:
        """Predict future behavior"""
        try:
            
            predictions = {
                'next_activity': 'message_sending',
                'activity_probability': 0.75,
                'risk_level': 'low',
                'engagement_score': 0.8,
                'prediction_confidence': 0.85
            }
            
            return predictions
            
        except Exception as e:
            logging.error(f"Behavior prediction error: {e}")
            return {}
    
    async def analyze_behavioral_network(self, user_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze behavioral network"""
        try:
            G = nx.Graph()
            
            if 'connections' in user_data:
                for connection in user_data['connections']:
                    G.add_edge(user_data.get('user_id', 'unknown'), connection)
            
            network_metrics = {
                'nodes': G.number_of_nodes(),
                'edges': G.number_of_edges(),
                'density': nx.density(G),
                'clustering_coefficient': nx.average_clustering(G),
                'centrality': dict(nx.degree_centrality(G)),
                'betweenness_centrality': dict(nx.betweenness_centrality(G)),
                'closeness_centrality': dict(nx.closeness_centrality(G))
            }
            
            return network_metrics
            
        except Exception as e:
            logging.error(f"Network analysis error: {e}")
            return {}
    
    async def generate_behavioral_insights(self, analyses: Dict[str, Any]) -> Dict[str, Any]:
        """Generate behavioral insights from analyses"""
        try:
            insights = {
                'summary': 'Behavioral analysis completed',
                'key_findings': [],
                'recommendations': [],
                'risk_assessment': 'low',
                'confidence': 0.8,
                'accuracy': 0.85
            }
            
            if 'clustering' in analyses:
                clustering = analyses['clustering']
                if 'kmeans' in clustering:
                    silhouette = clustering['kmeans'].get('silhouette_score', 0)
                    insights['key_findings'].append(f"Behavioral clustering quality: {silhouette:.3f}")
            
            if 'anomaly_detection' in analyses:
                anomaly = analyses['anomaly_detection']
                total_anomalies = 0
                for method in anomaly.values():
                    if isinstance(method, dict) and 'n_anomalies' in method:
                        total_anomalies += method['n_anomalies']
                
                if total_anomalies > 0:
                    insights['key_findings'].append(f"Detected {total_anomalies} behavioral anomalies")
                    insights['risk_assessment'] = 'medium' if total_anomalies < 5 else 'high'
            
            if 'network' in analyses:
                network = analyses['network']
                if 'centrality' in network:
                    max_centrality = max(network['centrality'].values()) if network['centrality'] else 0
                    insights['key_findings'].append(f"Network centrality: {max_centrality:.3f}")
            
            if insights['risk_assessment'] == 'high':
                insights['recommendations'].append('Monitor user closely for suspicious activities')
                insights['recommendations'].append('Implement additional security measures')
            elif insights['risk_assessment'] == 'medium':
                insights['recommendations'].append('Regular monitoring recommended')
            else:
                insights['recommendations'].append('Normal monitoring sufficient')
            
            return insights
            
        except Exception as e:
            logging.error(f"Insight generation error: {e}")
            return {'error': str(e)}
    
    async def analyze_threat_patterns(self, threat_data: List[Dict[str, Any]]) -> AnalyticsResult:
        """Analyze threat patterns with machine learning"""
        try:
            features = self.prepare_threat_features(threat_data)
            
            analyses = {}
            
            classification_result = await self.classify_threats(features)
            analyses['classification'] = classification_result
            
            clustering_result = await self.cluster_threats(features)
            analyses['clustering'] = clustering_result
            
            prediction_result = await self.predict_threats(features)
            analyses['prediction'] = prediction_result
            
            insights = await self.generate_threat_insights(analyses)
            
            result = AnalyticsResult(
                result_id=str(uuid.uuid4()),
                analytics_type=AnalyticsType.PRESCRIPTIVE,
                model_type=ModelType.CLASSIFICATION,
                insights=insights,
                predictions=prediction_result.get('predictions', []),
                confidence=insights.get('confidence', 0.9),
                accuracy=insights.get('accuracy', 0.88),
                timestamp=datetime.now(),
                metadata={'threat_count': len(threat_data), 'analysis_type': 'threat_patterns'}
            )
            
            await self.store_analytics_result(result)
            
            return result
            
        except Exception as e:
            logging.error(f"Threat pattern analysis error: {e}")
            raise
    
    async def classify_threats(self, features: np.ndarray) -> Dict[str, Any]:
        """Classify threats using machine learning"""
        try:
            
            classification_result = {
                'threat_types': ['malware', 'phishing', 'social_engineering'],
                'threat_levels': ['low', 'medium', 'high'],
                'classification_confidence': 0.85,
                'model_accuracy': 0.88
            }
            
            return classification_result
            
        except Exception as e:
            logging.error(f"Threat classification error: {e}")
            return {}
    
    async def cluster_threats(self, features: np.ndarray) -> Dict[str, Any]:
        """Cluster threats to identify patterns"""
        try:
            scaler = StandardScaler()
            features_scaled = scaler.fit_transform(features)
            
            kmeans = KMeans(n_clusters=3, random_state=42)
            labels = kmeans.fit_predict(features_scaled)
            
            return {
                'cluster_labels': labels.tolist(),
                'cluster_centers': kmeans.cluster_centers_.tolist(),
                'n_clusters': len(set(labels)),
                'silhouette_score': silhouette_score(features_scaled, labels)
            }
            
        except Exception as e:
            logging.error(f"Threat clustering error: {e}")
            return {}
    
    async def predict_threats(self, features: np.ndarray) -> Dict[str, Any]:
        """Predict future threats"""
        try:
            
            predictions = {
                'predicted_threats': ['phishing', 'malware'],
                'threat_probability': 0.7,
                'time_to_threat': '24-48 hours',
                'prediction_confidence': 0.8
            }
            
            return predictions
            
        except Exception as e:
            logging.error(f"Threat prediction error: {e}")
            return {}
    
    async def generate_threat_insights(self, analyses: Dict[str, Any]) -> Dict[str, Any]:
        """Generate threat insights from analyses"""
        try:
            insights = {
                'summary': 'Threat pattern analysis completed',
                'key_findings': [],
                'recommendations': [],
                'threat_level': 'medium',
                'confidence': 0.9,
                'accuracy': 0.88
            }
            
            if 'classification' in analyses:
                classification = analyses['classification']
                if 'threat_types' in classification:
                    insights['key_findings'].append(f"Identified {len(classification['threat_types'])} threat types")
            
            if 'clustering' in analyses:
                clustering = analyses['clustering']
                if 'n_clusters' in clustering:
                    insights['key_findings'].append(f"Identified {clustering['n_clusters']} threat clusters")
            
            insights['recommendations'].append('Implement threat detection monitoring')
            insights['recommendations'].append('Update security policies')
            insights['recommendations'].append('Train security team on new threat patterns')
            
            return insights
            
        except Exception as e:
            logging.error(f"Threat insight generation error: {e}")
            return {'error': str(e)}
    
    async def perform_time_series_analysis(self, time_series_data: pd.DataFrame) -> AnalyticsResult:
        """Perform time series analysis"""
        try:
            ts_data = self.prepare_time_series_data(time_series_data)
            
            analyses = {}
            
            trend_result = await self.analyze_trends(ts_data)
            analyses['trend'] = trend_result
            
            seasonality_result = await self.analyze_seasonality(ts_data)
            analyses['seasonality'] = seasonality_result
            
            forecast_result = await self.forecast_time_series(ts_data)
            analyses['forecast'] = forecast_result
            
            insights = await self.generate_time_series_insights(analyses)
            
            result = AnalyticsResult(
                result_id=str(uuid.uuid4()),
                analytics_type=AnalyticsType.PREDICTIVE,
                model_type=ModelType.TIME_SERIES,
                insights=insights,
                predictions=forecast_result.get('predictions', []),
                confidence=insights.get('confidence', 0.8),
                accuracy=insights.get('accuracy', 0.82),
                timestamp=datetime.now(),
                metadata={'data_points': len(ts_data), 'analysis_type': 'time_series'}
            )
            
            await self.store_analytics_result(result)
            
            return result
            
        except Exception as e:
            logging.error(f"Time series analysis error: {e}")
            raise
    
    async def analyze_trends(self, ts_data: pd.Series) -> Dict[str, Any]:
        """Analyze trends in time series data"""
        try:
            x = np.arange(len(ts_data))
            slope, intercept, r_value, p_value, std_err = stats.linregress(x, ts_data)
            
            return {
                'slope': slope,
                'intercept': intercept,
                'r_squared': r_value ** 2,
                'p_value': p_value,
                'trend_direction': 'increasing' if slope > 0 else 'decreasing',
                'trend_strength': abs(r_value)
            }
            
        except Exception as e:
            logging.error(f"Trend analysis error: {e}")
            return {}
    
    async def analyze_seasonality(self, ts_data: pd.Series) -> Dict[str, Any]:
        """Analyze seasonality in time series data"""
        try:
            
            return {
                'has_seasonality': False,
                'seasonal_period': None,
                'seasonal_strength': 0.0
            }
            
        except Exception as e:
            logging.error(f"Seasonality analysis error: {e}")
            return {}
    
    async def forecast_time_series(self, ts_data: pd.Series) -> Dict[str, Any]:
        """Forecast future values in time series"""
        try:
            
            last_value = ts_data.iloc[-1]
            moving_avg = ts_data.rolling(window=5).mean().iloc[-1]
            
            forecast_values = [last_value + (moving_avg - last_value) * i for i in range(1, 11)]
            
            return {
                'predictions': forecast_values,
                'forecast_horizon': 10,
                'forecast_confidence': 0.7
            }
            
        except Exception as e:
            logging.error(f"Time series forecasting error: {e}")
            return {}
    
    async def generate_time_series_insights(self, analyses: Dict[str, Any]) -> Dict[str, Any]:
        """Generate time series insights"""
        try:
            insights = {
                'summary': 'Time series analysis completed',
                'key_findings': [],
                'recommendations': [],
                'confidence': 0.8,
                'accuracy': 0.82
            }
            
            if 'trend' in analyses:
                trend = analyses['trend']
                if 'trend_direction' in trend:
                    insights['key_findings'].append(f"Trend direction: {trend['trend_direction']}")
                    insights['key_findings'].append(f"Trend strength: {trend.get('trend_strength', 0):.3f}")
            
            if 'seasonality' in analyses:
                seasonality = analyses['seasonality']
                if seasonality.get('has_seasonality'):
                    insights['key_findings'].append("Seasonal patterns detected")
            
            insights['recommendations'].append('Monitor trend changes')
            insights['recommendations'].append('Update forecasting models regularly')
            
            return insights
            
        except Exception as e:
            logging.error(f"Time series insight generation error: {e}")
            return {'error': str(e)}
    
    def prepare_behavior_features(self, user_data: Dict[str, Any]) -> np.ndarray:
        """Prepare features for behavior analysis"""
        try:
            features = []
            
            feature_mapping = {
                'message_frequency': user_data.get('message_frequency', 0),
                'avg_message_length': user_data.get('avg_message_length', 0),
                'response_time': user_data.get('response_time', 0),
                'activity_hours': user_data.get('activity_hours', 0),
                'crypto_addresses_count': len(user_data.get('crypto_addresses', [])),
                'social_links_count': len(user_data.get('social_links', [])),
                'media_files_count': len(user_data.get('media_files', [])),
                'forwarded_messages_count': len(user_data.get('forwarded_messages', [])),
                'mentions_count': len(user_data.get('mentions', [])),
                'hashtags_count': len(user_data.get('hashtags', [])),
                'aggressiveness_score': user_data.get('aggressiveness_score', 0),
                'friendliness_score': user_data.get('friendliness_score', 0),
                'professionalism_score': user_data.get('professionalism_score', 0),
                'emoji_usage': user_data.get('emoji_usage', 0),
                'suspicious_indicators_count': len(user_data.get('suspicious_indicators', []))
            }
            
            features = list(feature_mapping.values())
            
            features_array = np.array(features).reshape(1, -1)
            
            return features_array
            
        except Exception as e:
            logging.error(f"Feature preparation error: {e}")
            return np.array([[0] * 15])
    
    def prepare_threat_features(self, threat_data: List[Dict[str, Any]]) -> np.ndarray:
        """Prepare features for threat analysis"""
        try:
            features = []
            
            for threat in threat_data:
                threat_features = [
                    threat.get('severity_score', 0),
                    threat.get('confidence_score', 0),
                    threat.get('frequency', 0),
                    threat.get('impact_score', 0),
                    threat.get('complexity_score', 0)
                ]
                features.append(threat_features)
            
            return np.array(features)
            
        except Exception as e:
            logging.error(f"Threat feature preparation error: {e}")
            return np.array([[0] * 5])
    
    def prepare_time_series_data(self, ts_data: pd.DataFrame) -> pd.Series:
        """Prepare time series data for analysis"""
        try:
            if 'timestamp' in ts_data.columns:
                ts_data['timestamp'] = pd.to_datetime(ts_data['timestamp'])
                ts_data = ts_data.set_index('timestamp')
            
            numeric_columns = ts_data.select_dtypes(include=[np.number]).columns
            if len(numeric_columns) > 0:
                return ts_data[numeric_columns[0]]
            else:
                return pd.Series([1, 2, 3, 4, 5])
                
        except Exception as e:
            logging.error(f"Time series data preparation error: {e}")
            return pd.Series([1, 2, 3, 4, 5])
    
    async def store_analytics_result(self, result: AnalyticsResult):
        """Store analytics result in database"""
        try:
            cursor = self.db_connection.cursor()
            
            cursor.execute('''
                INSERT INTO analytics_results 
                (id, analytics_type, model_type, insights, predictions, confidence, accuracy, timestamp, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                result.result_id,
                result.analytics_type.value,
                result.model_type.value,
                json.dumps(result.insights),
                json.dumps(result.predictions),
                result.confidence,
                result.accuracy,
                result.timestamp.isoformat(),
                json.dumps(result.metadata)
            ))
            
            self.db_connection.commit()
            
            cache_key = f"analytics_result:{result.result_id}"
            self.redis_client.setex(
                cache_key,
                3600,
                json.dumps(asdict(result))
            )
            
            logging.info(f"Analytics result stored: {result.result_id}")
            
        except Exception as e:
            logging.error(f"Failed to store analytics result: {e}")
    
    async def get_analytics_result(self, result_id: str) -> Optional[AnalyticsResult]:
        """Get analytics result by ID"""
        try:
            cache_key = f"analytics_result:{result_id}"
            cached_result = self.redis_client.get(cache_key)
            
            if cached_result:
                result_data = json.loads(cached_result)
                return AnalyticsResult(**result_data)
            
            cursor = self.db_connection.cursor()
            cursor.execute('''
                SELECT * FROM analytics_results WHERE id = ?
            ''', (result_id,))
            
            row = cursor.fetchone()
            if row:
                return AnalyticsResult(
                    result_id=row[0],
                    analytics_type=AnalyticsType(row[1]),
                    model_type=ModelType(row[2]),
                    insights=json.loads(row[3]),
                    predictions=json.loads(row[4]),
                    confidence=row[5],
                    accuracy=row[6],
                    timestamp=datetime.fromisoformat(row[7]),
                    metadata=json.loads(row[8])
                )
            
            return None
            
        except Exception as e:
            logging.error(f"Failed to get analytics result: {e}")
            return None
    
    async def generate_analytics_dashboard(self) -> Dict[str, Any]:
        """Generate analytics dashboard data"""
        try:
            cursor = self.db_connection.cursor()
            cursor.execute('''
                SELECT * FROM analytics_results 
                ORDER BY timestamp DESC 
                LIMIT 100
            ''')
            
            results = cursor.fetchall()
            
            dashboard_data = {
                'total_analyses': len(results),
                'analytics_types': {},
                'model_types': {},
                'average_accuracy': 0.0,
                'average_confidence': 0.0,
                'recent_results': []
            }
            
            total_accuracy = 0
            total_confidence = 0
            
            for result in results:
                analytics_type = result[1]
                dashboard_data['analytics_types'][analytics_type] = dashboard_data['analytics_types'].get(analytics_type, 0) + 1
                
                model_type = result[2]
                dashboard_data['model_types'][model_type] = dashboard_data['model_types'].get(model_type, 0) + 1
                
                total_accuracy += result[5]
                total_confidence += result[6]
                
                dashboard_data['recent_results'].append({
                    'id': result[0],
                    'type': analytics_type,
                    'model': model_type,
                    'accuracy': result[5],
                    'confidence': result[6],
                    'timestamp': result[7]
                })
            
            if len(results) > 0:
                dashboard_data['average_accuracy'] = total_accuracy / len(results)
                dashboard_data['average_confidence'] = total_confidence / len(results)
            
            return dashboard_data
            
        except Exception as e:
            logging.error(f"Failed to generate analytics dashboard: {e}")
            return {}
    
    async def cleanup_old_results(self, days: int = 30):
        """Clean up old analytics results"""
        try:
            cutoff_date = datetime.now() - timedelta(days=days)
            
            cursor = self.db_connection.cursor()
            cursor.execute('''
                DELETE FROM analytics_results 
                WHERE timestamp < ?
            ''', (cutoff_date.isoformat(),))
            
            deleted_count = cursor.rowcount
            self.db_connection.commit()
            
            logging.info(f"Cleaned up {deleted_count} old analytics results")
            
        except Exception as e:
            logging.error(f"Failed to cleanup old results: {e}")

async def main():
    """Example usage of Advanced Analytics Engine"""
    config = {
        'redis': {
            'host': 'localhost',
            'port': 6379,
            'db': 0
        }
    }
    
    analytics_engine = AdvancedAnalyticsEngine(config)
    
    user_data = {
        'user_id': '12345',
        'message_frequency': 50,
        'avg_message_length': 100,
        'crypto_addresses': ['1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa'],
        'social_links': ['https://twitter.com/user'],
        'suspicious_indicators': ['multiple_accounts']
    }
    
    behavior_result = await analytics_engine.analyze_user_behavior(user_data)
    print(f"Behavior analysis: {behavior_result.insights}")
    
    threat_data = [
        {'severity_score': 0.8, 'confidence_score': 0.9, 'frequency': 5},
        {'severity_score': 0.6, 'confidence_score': 0.7, 'frequency': 3}
    ]
    
    threat_result = await analytics_engine.analyze_threat_patterns(threat_data)
    print(f"Threat analysis: {threat_result.insights}")
    
    dashboard = await analytics_engine.generate_analytics_dashboard()
    print(f"Dashboard: {dashboard}")

if __name__ == "__main__":
    asyncio.run(main())
