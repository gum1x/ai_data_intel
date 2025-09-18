"""
Advanced ML Models - Deep learning for behavior prediction and anomaly detection
"""

import numpy as np
import pandas as pd
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import tensorflow as tf
from sklearn.ensemble import IsolationForest, RandomForestClassifier
from sklearn.cluster import DBSCAN, KMeans
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report, confusion_matrix
import joblib
import pickle
from typing import Dict, List, Any, Tuple, Optional
import logging
from datetime import datetime, timedelta
import asyncio
from dataclasses import dataclass
from enum import Enum
import json

class ModelType(Enum):
    BEHAVIOR_PREDICTION = "behavior_prediction"
    ANOMALY_DETECTION = "anomaly_detection"
    THREAT_CLASSIFICATION = "threat_classification"
    USER_CLUSTERING = "user_clustering"
    ACTIVITY_FORECASTING = "activity_forecasting"
    SENTIMENT_ANALYSIS = "sentiment_analysis"

@dataclass
class ModelConfig:
    """Configuration for ML models"""
    model_type: ModelType
    input_features: List[str]
    output_classes: List[str]
    hidden_layers: List[int]
    learning_rate: float
    batch_size: int
    epochs: int
    dropout_rate: float
    regularization: float

class AdvancedMLModels:
    """Advanced ML models for intelligence analysis"""
    
    def __init__(self):
        self.models = {}
        self.scalers = {}
        self.encoders = {}
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        self.tf_device = '/GPU:0' if tf.config.list_physical_devices('GPU') else '/CPU:0'
        self.initialize_models()
    
    def initialize_models(self):
        """Initialize all ML models"""
        self.models[ModelType.BEHAVIOR_PREDICTION] = BehaviorPredictionModel()
        
        self.models[ModelType.ANOMALY_DETECTION] = AnomalyDetectionModel()
        
        self.models[ModelType.THREAT_CLASSIFICATION] = ThreatClassificationModel()
        
        self.models[ModelType.USER_CLUSTERING] = UserClusteringModel()
        
        self.models[ModelType.ACTIVITY_FORECASTING] = ActivityForecastingModel()
        
        self.models[ModelType.SENTIMENT_ANALYSIS] = SentimentAnalysisModel()
    
    async def train_behavior_prediction_model(self, training_data: pd.DataFrame) -> Dict[str, Any]:
        """Train behavior prediction model"""
        try:
            model = self.models[ModelType.BEHAVIOR_PREDICTION]
            
            features = self.prepare_behavior_features(training_data)
            labels = self.prepare_behavior_labels(training_data)
            
            X_train, X_test, y_train, y_test = train_test_split(
                features, labels, test_size=0.2, random_state=42
            )
            
            scaler = StandardScaler()
            X_train_scaled = scaler.fit_transform(X_train)
            X_test_scaled = scaler.transform(X_test)
            
            model.fit(X_train_scaled, y_train)
            
            predictions = model.predict(X_test_scaled)
            accuracy = model.score(X_test_scaled, y_test)
            
            self.scalers['behavior_prediction'] = scaler
            joblib.dump(model, 'models/behavior_prediction_model.pkl')
            
            return {
                'model_type': 'behavior_prediction',
                'accuracy': accuracy,
                'predictions': predictions.tolist(),
                'feature_importance': model.feature_importances_.tolist(),
                'training_samples': len(X_train),
                'test_samples': len(X_test)
            }
            
        except Exception as e:
            logging.error(f"Behavior prediction training error: {e}")
            return {'error': str(e)}
    
    async def train_anomaly_detection_model(self, training_data: pd.DataFrame) -> Dict[str, Any]:
        """Train anomaly detection model"""
        try:
            model = self.models[ModelType.ANOMALY_DETECTION]
            
            features = self.prepare_anomaly_features(training_data)
            
            model.fit(features)
            
            anomaly_scores = model.decision_function(features)
            predictions = model.predict(features)
            
            n_anomalies = sum(predictions == -1)
            anomaly_rate = n_anomalies / len(predictions)
            
            joblib.dump(model, 'models/anomaly_detection_model.pkl')
            
            return {
                'model_type': 'anomaly_detection',
                'anomaly_rate': anomaly_rate,
                'n_anomalies': n_anomalies,
                'n_normal': len(predictions) - n_anomalies,
                'anomaly_scores': anomaly_scores.tolist()
            }
            
        except Exception as e:
            logging.error(f"Anomaly detection training error: {e}")
            return {'error': str(e)}
    
    async def train_threat_classification_model(self, training_data: pd.DataFrame) -> Dict[str, Any]:
        """Train threat classification model"""
        try:
            model = self.models[ModelType.THREAT_CLASSIFICATION]
            
            features = self.prepare_threat_features(training_data)
            labels = self.prepare_threat_labels(training_data)
            
            X_train, X_test, y_train, y_test = train_test_split(
                features, labels, test_size=0.2, random_state=42
            )
            
            scaler = StandardScaler()
            X_train_scaled = scaler.fit_transform(X_train)
            X_test_scaled = scaler.transform(X_test)
            
            model.fit(X_train_scaled, y_train)
            
            predictions = model.predict(X_test_scaled)
            accuracy = model.score(X_test_scaled, y_test)
            
            report = classification_report(y_test, predictions, output_dict=True)
            
            self.scalers['threat_classification'] = scaler
            joblib.dump(model, 'models/threat_classification_model.pkl')
            
            return {
                'model_type': 'threat_classification',
                'accuracy': accuracy,
                'classification_report': report,
                'predictions': predictions.tolist(),
                'feature_importance': model.feature_importances_.tolist()
            }
            
        except Exception as e:
            logging.error(f"Threat classification training error: {e}")
            return {'error': str(e)}
    
    async def train_user_clustering_model(self, training_data: pd.DataFrame) -> Dict[str, Any]:
        """Train user clustering model"""
        try:
            model = self.models[ModelType.USER_CLUSTERING]
            
            features = self.prepare_clustering_features(training_data)
            
            scaler = StandardScaler()
            features_scaled = scaler.fit_transform(features)
            
            model.fit(features_scaled)
            
            cluster_labels = model.labels_
            n_clusters = len(set(cluster_labels)) - (1 if -1 in cluster_labels else 0)
            
            cluster_stats = {}
            for cluster_id in range(n_clusters):
                cluster_mask = cluster_labels == cluster_id
                cluster_data = features[cluster_mask]
                cluster_stats[cluster_id] = {
                    'size': len(cluster_data),
                    'centroid': cluster_data.mean(axis=0).tolist(),
                    'std': cluster_data.std(axis=0).tolist()
                }
            
            self.scalers['user_clustering'] = scaler
            joblib.dump(model, 'models/user_clustering_model.pkl')
            
            return {
                'model_type': 'user_clustering',
                'n_clusters': n_clusters,
                'cluster_labels': cluster_labels.tolist(),
                'cluster_stats': cluster_stats,
                'silhouette_score': self.calculate_silhouette_score(features_scaled, cluster_labels)
            }
            
        except Exception as e:
            logging.error(f"User clustering training error: {e}")
            return {'error': str(e)}
    
    async def train_activity_forecasting_model(self, training_data: pd.DataFrame) -> Dict[str, Any]:
        """Train activity forecasting model"""
        try:
            model = self.models[ModelType.ACTIVITY_FORECASTING]
            
            time_series_data = self.prepare_time_series_data(training_data)
            
            history = model.train_lstm(time_series_data)
            
            predictions = model.predict_next_24_hours(time_series_data)
            
            mse = model.calculate_mse(time_series_data, predictions)
            mae = model.calculate_mae(time_series_data, predictions)
            
            model.save_model('models/activity_forecasting_model.h5')
            
            return {
                'model_type': 'activity_forecasting',
                'mse': mse,
                'mae': mae,
                'predictions': predictions.tolist(),
                'training_history': history.history
            }
            
        except Exception as e:
            logging.error(f"Activity forecasting training error: {e}")
            return {'error': str(e)}
    
    async def train_sentiment_analysis_model(self, training_data: pd.DataFrame) -> Dict[str, Any]:
        """Train sentiment analysis model"""
        try:
            model = self.models[ModelType.SENTIMENT_ANALYSIS]
            
            texts = training_data['text'].tolist()
            labels = training_data['sentiment'].tolist()
            
            history = model.train_transformer(texts, labels)
            
            test_texts = texts[-100:]
            test_labels = labels[-100:]
            predictions = model.predict_sentiment(test_texts)
            
            accuracy = sum(1 for p, l in zip(predictions, test_labels) if p == l) / len(predictions)
            
            model.save_model('models/sentiment_analysis_model.h5')
            
            return {
                'model_type': 'sentiment_analysis',
                'accuracy': accuracy,
                'predictions': predictions,
                'training_history': history.history
            }
            
        except Exception as e:
            logging.error(f"Sentiment analysis training error: {e}")
            return {'error': str(e)}
    
    def prepare_behavior_features(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare features for behavior prediction"""
        features = []
        
        for _, row in data.iterrows():
            feature_vector = [
                row.get('message_frequency', 0),
                row.get('avg_message_length', 0),
                row.get('response_time', 0),
                row.get('activity_hours', 0),
                row.get('crypto_addresses_count', 0),
                row.get('social_links_count', 0),
                row.get('media_files_count', 0),
                row.get('forwarded_messages_count', 0),
                row.get('mentions_count', 0),
                row.get('hashtags_count', 0),
                row.get('aggressiveness_score', 0),
                row.get('friendliness_score', 0),
                row.get('professionalism_score', 0),
                row.get('emoji_usage', 0),
                row.get('suspicious_indicators_count', 0)
            ]
            features.append(feature_vector)
        
        return np.array(features)
    
    def prepare_behavior_labels(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare labels for behavior prediction"""
        labels = []
        
        for _, row in data.iterrows():
            if row.get('suspicious_indicators_count', 0) > 5:
                label = 'suspicious'
            elif row.get('crypto_addresses_count', 0) > 2:
                label = 'crypto_enthusiast'
            elif row.get('social_links_count', 0) > 3:
                label = 'social_media_active'
            elif row.get('message_frequency', 0) > 50:
                label = 'highly_active'
            else:
                label = 'normal'
            
            labels.append(label)
        
        return np.array(labels)
    
    def prepare_anomaly_features(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare features for anomaly detection"""
        features = []
        
        for _, row in data.iterrows():
            feature_vector = [
                row.get('message_frequency', 0),
                row.get('avg_message_length', 0),
                row.get('crypto_addresses_count', 0),
                row.get('social_links_count', 0),
                row.get('media_files_count', 0),
                row.get('forwarded_messages_count', 0),
                row.get('suspicious_indicators_count', 0),
                row.get('aggressiveness_score', 0),
                row.get('emoji_usage', 0),
                row.get('common_chats_count', 0)
            ]
            features.append(feature_vector)
        
        return np.array(features)
    
    def prepare_threat_features(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare features for threat classification"""
        features = []
        
        for _, row in data.iterrows():
            feature_vector = [
                row.get('suspicious_indicators_count', 0),
                row.get('crypto_addresses_count', 0),
                row.get('aggressiveness_score', 0),
                row.get('message_frequency', 0),
                row.get('forwarded_messages_count', 0),
                row.get('media_files_count', 0),
                row.get('social_links_count', 0),
                row.get('mentions_count', 0),
                row.get('hashtags_count', 0),
                row.get('emoji_usage', 0)
            ]
            features.append(feature_vector)
        
        return np.array(features)
    
    def prepare_threat_labels(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare labels for threat classification"""
        labels = []
        
        for _, row in data.iterrows():
            if row.get('suspicious_indicators_count', 0) > 8:
                label = 'high_threat'
            elif row.get('suspicious_indicators_count', 0) > 4:
                label = 'medium_threat'
            elif row.get('suspicious_indicators_count', 0) > 1:
                label = 'low_threat'
            else:
                label = 'no_threat'
            
            labels.append(label)
        
        return np.array(labels)
    
    def prepare_clustering_features(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare features for user clustering"""
        features = []
        
        for _, row in data.iterrows():
            feature_vector = [
                row.get('message_frequency', 0),
                row.get('avg_message_length', 0),
                row.get('crypto_addresses_count', 0),
                row.get('social_links_count', 0),
                row.get('media_files_count', 0),
                row.get('forwarded_messages_count', 0),
                row.get('mentions_count', 0),
                row.get('hashtags_count', 0),
                row.get('aggressiveness_score', 0),
                row.get('friendliness_score', 0),
                row.get('professionalism_score', 0),
                row.get('emoji_usage', 0),
                row.get('common_chats_count', 0)
            ]
            features.append(feature_vector)
        
        return np.array(features)
    
    def prepare_time_series_data(self, data: pd.DataFrame) -> np.ndarray:
        """Prepare time series data for activity forecasting"""
        data['timestamp'] = pd.to_datetime(data['timestamp'])
        data['hour'] = data['timestamp'].dt.hour
        
        hourly_activity = data.groupby('hour').size().values
        
        if len(hourly_activity) < 24:
            padded_activity = np.zeros(24)
            padded_activity[:len(hourly_activity)] = hourly_activity
            return padded_activity
        
        return hourly_activity
    
    def calculate_silhouette_score(self, features: np.ndarray, labels: np.ndarray) -> float:
        """Calculate silhouette score for clustering"""
        from sklearn.metrics import silhouette_score
        try:
            return silhouette_score(features, labels)
        except:
            return 0.0

class BehaviorPredictionModel:
    """Behavior prediction model using Random Forest"""
    
    def __init__(self):
        self.model = RandomForestClassifier(
            n_estimators=100,
            max_depth=10,
            random_state=42
        )
    
    def fit(self, X, y):
        self.model.fit(X, y)
    
    def predict(self, X):
        return self.model.predict(X)
    
    def score(self, X, y):
        return self.model.score(X, y)
    
    @property
    def feature_importances_(self):
        return self.model.feature_importances_

class AnomalyDetectionModel:
    """Anomaly detection model using Isolation Forest"""
    
    def __init__(self):
        self.model = IsolationForest(
            contamination=0.1,
            random_state=42
        )
    
    def fit(self, X):
        self.model.fit(X)
    
    def predict(self, X):
        return self.model.predict(X)
    
    def decision_function(self, X):
        return self.model.decision_function(X)

class ThreatClassificationModel:
    """Threat classification model using Random Forest"""
    
    def __init__(self):
        self.model = RandomForestClassifier(
            n_estimators=200,
            max_depth=15,
            random_state=42
        )
    
    def fit(self, X, y):
        self.model.fit(X, y)
    
    def predict(self, X):
        return self.model.predict(X)
    
    def score(self, X, y):
        return self.model.score(X, y)
    
    @property
    def feature_importances_(self):
        return self.model.feature_importances_

class UserClusteringModel:
    """User clustering model using DBSCAN"""
    
    def __init__(self):
        self.model = DBSCAN(
            eps=0.5,
            min_samples=5
        )
    
    def fit(self, X):
        self.model.fit(X)
    
    @property
    def labels_(self):
        return self.model.labels_

class ActivityForecastingModel:
    """Activity forecasting model using LSTM"""
    
    def __init__(self):
        self.model = None
        self.scaler = StandardScaler()
    
    def train_lstm(self, time_series_data: np.ndarray) -> Any:
        """Train LSTM model for activity forecasting"""
        X, y = self.prepare_lstm_data(time_series_data)
        
        X_scaled = self.scaler.fit_transform(X)
        y_scaled = self.scaler.transform(y.reshape(-1, 1))
        
        self.model = tf.keras.Sequential([
            tf.keras.layers.LSTM(50, return_sequences=True, input_shape=(X.shape[1], 1)),
            tf.keras.layers.LSTM(50, return_sequences=False),
            tf.keras.layers.Dense(25),
            tf.keras.layers.Dense(1)
        ])
        
        self.model.compile(optimizer='adam', loss='mean_squared_error')
        
        history = self.model.fit(
            X_scaled, y_scaled,
            batch_size=1,
            epochs=100,
            validation_split=0.2,
            verbose=0
        )
        
        return history
    
    def prepare_lstm_data(self, time_series_data: np.ndarray, look_back: int = 24) -> Tuple[np.ndarray, np.ndarray]:
        """Prepare data for LSTM training"""
        X, y = [], []
        
        for i in range(look_back, len(time_series_data)):
            X.append(time_series_data[i-look_back:i])
            y.append(time_series_data[i])
        
        return np.array(X), np.array(y)
    
    def predict_next_24_hours(self, time_series_data: np.ndarray) -> np.ndarray:
        """Predict next 24 hours of activity"""
        if self.model is None:
            return np.zeros(24)
        
        last_24_hours = time_series_data[-24:].reshape(1, 24, 1)
        predictions = []
        
        for _ in range(24):
            pred = self.model.predict(last_24_hours, verbose=0)
            predictions.append(pred[0, 0])
            
            last_24_hours = np.roll(last_24_hours, -1, axis=1)
            last_24_hours[0, -1, 0] = pred[0, 0]
        
        return np.array(predictions)
    
    def calculate_mse(self, actual: np.ndarray, predicted: np.ndarray) -> float:
        """Calculate Mean Squared Error"""
        return np.mean((actual - predicted) ** 2)
    
    def calculate_mae(self, actual: np.ndarray, predicted: np.ndarray) -> float:
        """Calculate Mean Absolute Error"""
        return np.mean(np.abs(actual - predicted))
    
    def save_model(self, filepath: str):
        """Save the model"""
        if self.model:
            self.model.save(filepath)

class SentimentAnalysisModel:
    """Sentiment analysis model using Transformer"""
    
    def __init__(self):
        self.model = None
        self.tokenizer = None
    
    def train_transformer(self, texts: List[str], labels: List[str]) -> Any:
        """Train transformer model for sentiment analysis"""
        
        tokenizer = tf.keras.preprocessing.text.Tokenizer(num_words=10000)
        tokenizer.fit_on_texts(texts)
        
        sequences = tokenizer.texts_to_sequences(texts)
        padded_sequences = tf.keras.preprocessing.sequence.pad_sequences(sequences, maxlen=100)
        
        label_encoder = LabelEncoder()
        encoded_labels = label_encoder.fit_transform(labels)
        
        self.model = tf.keras.Sequential([
            tf.keras.layers.Embedding(10000, 128, input_length=100),
            tf.keras.layers.LSTM(64, return_sequences=True),
            tf.keras.layers.LSTM(32),
            tf.keras.layers.Dense(64, activation='relu'),
            tf.keras.layers.Dropout(0.5),
            tf.keras.layers.Dense(len(set(labels)), activation='softmax')
        ])
        
        self.model.compile(
            optimizer='adam',
            loss='sparse_categorical_crossentropy',
            metrics=['accuracy']
        )
        
        history = self.model.fit(
            padded_sequences, encoded_labels,
            batch_size=32,
            epochs=10,
            validation_split=0.2,
            verbose=0
        )
        
        self.tokenizer = tokenizer
        self.label_encoder = label_encoder
        
        return history
    
    def predict_sentiment(self, texts: List[str]) -> List[str]:
        """Predict sentiment for texts"""
        if self.model is None or self.tokenizer is None:
            return ['neutral'] * len(texts)
        
        sequences = self.tokenizer.texts_to_sequences(texts)
        padded_sequences = tf.keras.preprocessing.sequence.pad_sequences(sequences, maxlen=100)
        
        predictions = self.model.predict(padded_sequences, verbose=0)
        predicted_labels = np.argmax(predictions, axis=1)
        
        return [self.label_encoder.inverse_transform([label])[0] for label in predicted_labels]
    
    def save_model(self, filepath: str):
        """Save the model"""
        if self.model:
            self.model.save(filepath)

async def main():
    """Example usage of AdvancedMLModels"""
    ml_models = AdvancedMLModels()
    
    sample_data = pd.DataFrame({
        'message_frequency': np.random.randint(1, 100, 1000),
        'avg_message_length': np.random.randint(10, 500, 1000),
        'crypto_addresses_count': np.random.randint(0, 5, 1000),
        'social_links_count': np.random.randint(0, 10, 1000),
        'suspicious_indicators_count': np.random.randint(0, 10, 1000),
        'aggressiveness_score': np.random.random(1000),
        'friendliness_score': np.random.random(1000),
        'professionalism_score': np.random.random(1000),
        'emoji_usage': np.random.random(1000),
        'common_chats_count': np.random.randint(0, 50, 1000),
        'timestamp': pd.date_range('2023-01-01', periods=1000, freq='H'),
        'text': ['Sample text'] * 1000,
        'sentiment': np.random.choice(['positive', 'negative', 'neutral'], 1000)
    })
    
    print("Training behavior prediction model...")
    behavior_results = await ml_models.train_behavior_prediction_model(sample_data)
    print(f"Behavior prediction accuracy: {behavior_results.get('accuracy', 0):.3f}")
    
    print("Training anomaly detection model...")
    anomaly_results = await ml_models.train_anomaly_detection_model(sample_data)
    print(f"Anomaly rate: {anomaly_results.get('anomaly_rate', 0):.3f}")
    
    print("Training threat classification model...")
    threat_results = await ml_models.train_threat_classification_model(sample_data)
    print(f"Threat classification accuracy: {threat_results.get('accuracy', 0):.3f}")
    
    print("Training user clustering model...")
    clustering_results = await ml_models.train_user_clustering_model(sample_data)
    print(f"Number of clusters: {clustering_results.get('n_clusters', 0)}")
    
    print("Training activity forecasting model...")
    forecasting_results = await ml_models.train_activity_forecasting_model(sample_data)
    print(f"Activity forecasting MSE: {forecasting_results.get('mse', 0):.3f}")
    
    print("Training sentiment analysis model...")
    sentiment_results = await ml_models.train_sentiment_analysis_model(sample_data)
    print(f"Sentiment analysis accuracy: {sentiment_results.get('accuracy', 0):.3f}")

if __name__ == "__main__":
    asyncio.run(main())
