"""
Production-Ready PostgreSQL Database Manager
Replaces SQLite with enterprise-grade PostgreSQL infrastructure
"""

import asyncio
import asyncpg
import logging
import json
import uuid
from datetime import datetime, timezone
from typing import Dict, List, Any, Optional, Union, Tuple
from dataclasses import dataclass, asdict
from contextlib import asynccontextmanager
import redis.asyncio as redis
from sqlalchemy import create_engine, text, MetaData, Table, Column, String, Integer, Float, DateTime, Boolean, JSON, Text, Index
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy.dialects.postgresql import UUID, ARRAY, JSONB
from sqlalchemy.pool import QueuePool
import os
from enum import Enum

# Database Models
Base = declarative_base()

class DataSource(Enum):
    TELEGRAM = "telegram"
    BLOCKCHAIN = "blockchain"
    SOCIAL_MEDIA = "social_media"
    WEB_SCRAPING = "web_scraping"
    API = "api"
    MANUAL = "manual"

class DataClassification(Enum):
    PUBLIC = "public"
    INTERNAL = "internal"
    CONFIDENTIAL = "confidential"
    SECRET = "secret"

class ThreatLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

class AgentStatus(Enum):
    ACTIVE = "active"
    INACTIVE = "inactive"
    ERROR = "error"
    MAINTENANCE = "maintenance"

# Core Tables
class IntelligenceData(Base):
    __tablename__ = "intelligence_data"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    content = Column(JSONB, nullable=False)
    source = Column(String(50), nullable=False)
    classification = Column(String(20), nullable=False)
    metadata = Column(JSONB, default={})
    timestamp = Column(DateTime(timezone=True), default=datetime.utcnow)
    confidence = Column(Float, default=1.0)
    quality_score = Column(Float, default=1.0)
    hash = Column(String(64), unique=True)
    signatures = Column(ARRAY(String))
    created_at = Column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at = Column(DateTime(timezone=True), default=datetime.utcnow, onupdate=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_intelligence_source', 'source'),
        Index('idx_intelligence_classification', 'classification'),
        Index('idx_intelligence_timestamp', 'timestamp'),
        Index('idx_intelligence_confidence', 'confidence'),
        Index('idx_intelligence_hash', 'hash'),
    )

class UserProfile(Base):
    __tablename__ = "user_profiles"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id = Column(String(100), unique=True, nullable=False)
    username = Column(String(100))
    first_name = Column(String(100))
    last_name = Column(String(100))
    phone = Column(String(20))
    bio = Column(Text)
    profile_photo_id = Column(String(100))
    is_verified = Column(Boolean, default=False)
    is_premium = Column(Boolean, default=False)
    is_bot = Column(Boolean, default=False)
    is_scam = Column(Boolean, default=False)
    is_fake = Column(Boolean, default=False)
    common_chats_count = Column(Integer, default=0)
    last_seen = Column(DateTime(timezone=True))
    behavioral_score = Column(Float, default=0.5)
    risk_level = Column(String(20), default="low")
    suspicious_indicators = Column(ARRAY(String))
    network_connections = Column(ARRAY(String))
    created_at = Column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at = Column(DateTime(timezone=True), default=datetime.utcnow, onupdate=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_user_profile_user_id', 'user_id'),
        Index('idx_user_profile_username', 'username'),
        Index('idx_user_profile_risk_level', 'risk_level'),
        Index('idx_user_profile_behavioral_score', 'behavioral_score'),
    )

class ThreatAssessment(Base):
    __tablename__ = "threat_assessments"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id = Column(String(100), nullable=False)
    threat_type = Column(String(50), nullable=False)
    threat_level = Column(String(20), nullable=False)
    confidence = Column(Float, nullable=False)
    indicators = Column(ARRAY(String))
    description = Column(Text)
    recommendations = Column(ARRAY(String))
    status = Column(String(20), default="active")
    assigned_to = Column(String(100))
    resolved_at = Column(DateTime(timezone=True))
    created_at = Column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at = Column(DateTime(timezone=True), default=datetime.utcnow, onupdate=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_threat_user_id', 'user_id'),
        Index('idx_threat_level', 'threat_level'),
        Index('idx_threat_type', 'threat_type'),
        Index('idx_threat_status', 'status'),
        Index('idx_threat_created_at', 'created_at'),
    )

class AgentTask(Base):
    __tablename__ = "agent_tasks"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    task_type = Column(String(50), nullable=False)
    priority = Column(Integer, default=1)
    status = Column(String(20), default="pending")
    assigned_agent = Column(String(100))
    parameters = Column(JSONB)
    result = Column(JSONB)
    error_message = Column(Text)
    started_at = Column(DateTime(timezone=True))
    completed_at = Column(DateTime(timezone=True))
    created_at = Column(DateTime(timezone=True), default=datetime.utcnow)
    updated_at = Column(DateTime(timezone=True), default=datetime.utcnow, onupdate=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_task_type', 'task_type'),
        Index('idx_task_status', 'status'),
        Index('idx_task_priority', 'priority'),
        Index('idx_task_assigned_agent', 'assigned_agent'),
    )

class SystemMetrics(Base):
    __tablename__ = "system_metrics"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    metric_name = Column(String(100), nullable=False)
    metric_value = Column(Float, nullable=False)
    metric_type = Column(String(50), nullable=False)
    tags = Column(JSONB, default={})
    timestamp = Column(DateTime(timezone=True), default=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_metrics_name', 'metric_name'),
        Index('idx_metrics_type', 'metric_type'),
        Index('idx_metrics_timestamp', 'timestamp'),
    )

class AuditLog(Base):
    __tablename__ = "audit_logs"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id = Column(String(100))
    action = Column(String(100), nullable=False)
    resource = Column(String(100), nullable=False)
    resource_id = Column(String(100))
    details = Column(JSONB)
    ip_address = Column(String(45))
    user_agent = Column(String(500))
    timestamp = Column(DateTime(timezone=True), default=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_audit_user_id', 'user_id'),
        Index('idx_audit_action', 'action'),
        Index('idx_audit_resource', 'resource'),
        Index('idx_audit_timestamp', 'timestamp'),
    )

class Conversation(Base):
    __tablename__ = "conversations"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    session_id = Column(String(100), nullable=False)
    provider = Column(String(50), nullable=False)
    conversation_type = Column(String(50), nullable=False)
    question = Column(Text, nullable=False)
    response = Column(Text, nullable=False)
    context = Column(JSONB)
    timestamp = Column(DateTime(timezone=True), default=datetime.utcnow)
    tab_id = Column(String(100))
    message_number = Column(Integer)
    
    __table_args__ = (
        Index('idx_conversation_session_id', 'session_id'),
        Index('idx_conversation_provider', 'provider'),
        Index('idx_conversation_type', 'conversation_type'),
        Index('idx_conversation_timestamp', 'timestamp'),
    )

class AIInsight(Base):
    __tablename__ = "ai_insights"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    insight_type = Column(String(50), nullable=False)
    content = Column(Text, nullable=False)
    confidence = Column(Float, nullable=False)
    source_conversation = Column(String(100))
    metadata = Column(JSONB)
    timestamp = Column(DateTime(timezone=True), default=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_insight_type', 'insight_type'),
        Index('idx_insight_confidence', 'confidence'),
        Index('idx_insight_timestamp', 'timestamp'),
    )

class DatabaseManager:
    """Production-ready PostgreSQL database manager"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.engine = None
        self.session_factory = None
        self.redis_client = None
        self.connection_pool = None
        self.is_initialized = False
        
        # Database configuration
        self.db_config = config.get('database', {})
        self.redis_config = config.get('cache', {})
        
        # Connection parameters
        self.database_url = self._build_database_url()
        self.redis_url = self._build_redis_url()
        
        # Performance settings
        self.pool_size = self.db_config.get('max_connections', 100)
        self.max_overflow = self.pool_size * 2
        self.pool_timeout = self.db_config.get('connection_timeout', 30)
        self.pool_recycle = 3600
        
    def _build_database_url(self) -> str:
        """Build PostgreSQL connection URL"""
        host = os.getenv('DATABASE_HOST', self.db_config.get('host', 'localhost'))
        port = os.getenv('DATABASE_PORT', self.db_config.get('port', 5432))
        name = os.getenv('DATABASE_NAME', self.db_config.get('name', 'intelligence'))
        username = os.getenv('DATABASE_USER', self.db_config.get('username', 'intelligence'))
        password = os.getenv('DATABASE_PASSWORD', self.db_config.get('password', 'password'))
        
        return f"postgresql://{username}:{password}@{host}:{port}/{name}"
    
    def _build_redis_url(self) -> str:
        """Build Redis connection URL"""
        host = os.getenv('REDIS_HOST', self.redis_config.get('host', 'localhost'))
        port = os.getenv('REDIS_PORT', self.redis_config.get('port', 6379))
        password = os.getenv('REDIS_PASSWORD', self.redis_config.get('password'))
        database = os.getenv('REDIS_DB', self.redis_config.get('database', 0))
        
        if password:
            return f"redis://:{password}@{host}:{port}/{database}"
        return f"redis://{host}:{port}/{database}"
    
    async def initialize(self):
        """Initialize database connections and create tables"""
        try:
            # Initialize SQLAlchemy engine
            self.engine = create_engine(
                self.database_url,
                poolclass=QueuePool,
                pool_size=self.pool_size,
                max_overflow=self.max_overflow,
                pool_timeout=self.pool_timeout,
                pool_recycle=self.pool_recycle,
                pool_pre_ping=True,
                echo=False
            )
            
            # Create session factory
            self.session_factory = sessionmaker(
                bind=self.engine,
                autocommit=False,
                autoflush=False
            )
            
            # Initialize Redis
            self.redis_client = redis.from_url(
                self.redis_url,
                encoding="utf-8",
                decode_responses=True,
                max_connections=50
            )
            
            # Create tables
            await self.create_tables()
            
            # Test connections
            await self.test_connections()
            
            self.is_initialized = True
            logging.info("Database manager initialized successfully")
            
        except Exception as e:
            logging.error(f"Failed to initialize database manager: {e}")
            raise
    
    async def create_tables(self):
        """Create all database tables"""
        try:
            # Create all tables
            Base.metadata.create_all(self.engine)
            
            # Create additional indexes for performance
            await self.create_additional_indexes()
            
            logging.info("Database tables created successfully")
            
        except Exception as e:
            logging.error(f"Failed to create tables: {e}")
            raise
    
    async def create_additional_indexes(self):
        """Create additional performance indexes"""
        try:
            with self.engine.connect() as conn:
                # Composite indexes for common queries
                indexes = [
                    "CREATE INDEX IF NOT EXISTS idx_intelligence_source_timestamp ON intelligence_data (source, timestamp)",
                    "CREATE INDEX IF NOT EXISTS idx_intelligence_classification_confidence ON intelligence_data (classification, confidence)",
                    "CREATE INDEX IF NOT EXISTS idx_user_profile_risk_behavioral ON user_profiles (risk_level, behavioral_score)",
                    "CREATE INDEX IF NOT EXISTS idx_threat_level_created ON threat_assessments (threat_level, created_at)",
                    "CREATE INDEX IF NOT EXISTS idx_task_status_priority ON agent_tasks (status, priority)",
                    "CREATE INDEX IF NOT EXISTS idx_metrics_name_timestamp ON system_metrics (metric_name, timestamp)",
                    "CREATE INDEX IF NOT EXISTS idx_audit_user_timestamp ON audit_logs (user_id, timestamp)",
                ]
                
                for index_sql in indexes:
                    conn.execute(text(index_sql))
                
                conn.commit()
                
        except Exception as e:
            logging.error(f"Failed to create additional indexes: {e}")
            raise
    
    async def test_connections(self):
        """Test database and Redis connections"""
        try:
            # Test PostgreSQL connection
            with self.session_factory() as session:
                result = session.execute(text("SELECT 1"))
                assert result.scalar() == 1
            
            # Test Redis connection
            await self.redis_client.ping()
            
            logging.info("Database connections tested successfully")
            
        except Exception as e:
            logging.error(f"Database connection test failed: {e}")
            raise
    
    @asynccontextmanager
    async def get_session(self):
        """Get database session with automatic cleanup"""
        session = self.session_factory()
        try:
            yield session
            session.commit()
        except Exception:
            session.rollback()
            raise
        finally:
            session.close()
    
    async def store_intelligence_data(self, data: Dict[str, Any]) -> str:
        """Store intelligence data"""
        try:
            async with self.get_session() as session:
                intelligence_data = IntelligenceData(
                    content=data.get('content', {}),
                    source=data.get('source', 'unknown'),
                    classification=data.get('classification', 'internal'),
                    metadata=data.get('metadata', {}),
                    timestamp=data.get('timestamp', datetime.utcnow()),
                    confidence=data.get('confidence', 1.0),
                    quality_score=data.get('quality_score', 1.0),
                    hash=data.get('hash', ''),
                    signatures=data.get('signatures', [])
                )
                
                session.add(intelligence_data)
                session.flush()
                
                # Cache in Redis
                await self.redis_client.setex(
                    f"intelligence_data:{intelligence_data.id}",
                    3600,  # 1 hour TTL
                    json.dumps(asdict(intelligence_data), default=str)
                )
                
                return str(intelligence_data.id)
                
        except Exception as e:
            logging.error(f"Failed to store intelligence data: {e}")
            raise
    
    async def get_intelligence_data(self, data_id: str) -> Optional[Dict[str, Any]]:
        """Get intelligence data by ID"""
        try:
            # Try Redis cache first
            cached_data = await self.redis_client.get(f"intelligence_data:{data_id}")
            if cached_data:
                return json.loads(cached_data)
            
            # Fallback to database
            async with self.get_session() as session:
                result = session.query(IntelligenceData).filter(
                    IntelligenceData.id == data_id
                ).first()
                
                if result:
                    data = asdict(result)
                    # Cache for future requests
                    await self.redis_client.setex(
                        f"intelligence_data:{data_id}",
                        3600,
                        json.dumps(data, default=str)
                    )
                    return data
                
                return None
                
        except Exception as e:
            logging.error(f"Failed to get intelligence data: {e}")
            raise
    
    async def store_user_profile(self, profile_data: Dict[str, Any]) -> str:
        """Store user profile"""
        try:
            async with self.get_session() as session:
                user_profile = UserProfile(
                    user_id=profile_data.get('user_id'),
                    username=profile_data.get('username'),
                    first_name=profile_data.get('first_name'),
                    last_name=profile_data.get('last_name'),
                    phone=profile_data.get('phone'),
                    bio=profile_data.get('bio'),
                    profile_photo_id=profile_data.get('profile_photo_id'),
                    is_verified=profile_data.get('is_verified', False),
                    is_premium=profile_data.get('is_premium', False),
                    is_bot=profile_data.get('is_bot', False),
                    is_scam=profile_data.get('is_scam', False),
                    is_fake=profile_data.get('is_fake', False),
                    common_chats_count=profile_data.get('common_chats_count', 0),
                    last_seen=profile_data.get('last_seen'),
                    behavioral_score=profile_data.get('behavioral_score', 0.5),
                    risk_level=profile_data.get('risk_level', 'low'),
                    suspicious_indicators=profile_data.get('suspicious_indicators', [])
                )
                
                session.add(user_profile)
                session.flush()
                
                # Cache in Redis
                await self.redis_client.setex(
                    f"user_profile:{user_profile.user_id}",
                    1800,  # 30 minutes TTL
                    json.dumps(asdict(user_profile), default=str)
                )
                
                return str(user_profile.id)
                
        except Exception as e:
            logging.error(f"Failed to store user profile: {e}")
            raise
    
    async def store_threat_alert(self, threat_data: Dict[str, Any]) -> str:
        """Store threat alert"""
        try:
            async with self.get_session() as session:
                threat_assessment = ThreatAssessment(
                    user_id=threat_data.get('user_id'),
                    threat_type=threat_data.get('threat_type', 'unknown'),
                    threat_level=threat_data.get('threat_level', 'low'),
                    confidence=threat_data.get('confidence', 0.5),
                    indicators=threat_data.get('indicators', []),
                    description=threat_data.get('description'),
                    recommendations=threat_data.get('recommendations', []),
                    assigned_to=threat_data.get('assigned_to')
                )
                
                session.add(threat_assessment)
                session.flush()
                
                # Cache in Redis
                await self.redis_client.setex(
                    f"threat_alert:{threat_assessment.id}",
                    3600,  # 1 hour TTL
                    json.dumps(asdict(threat_assessment), default=str)
                )
                
                return str(threat_assessment.id)
                
        except Exception as e:
            logging.error(f"Failed to store threat alert: {e}")
            raise
    
    async def store_ai_insight(self, insight_data: Dict[str, Any]) -> str:
        """Store AI insight"""
        try:
            async with self.get_session() as session:
                ai_insight = AIInsight(
                    insight_type=insight_data.get('type', 'general'),
                    content=insight_data.get('content', ''),
                    confidence=insight_data.get('confidence', 0.5),
                    source_conversation=insight_data.get('source_conversation'),
                    metadata=insight_data.get('metadata', {})
                )
                
                session.add(ai_insight)
                session.flush()
                
                # Cache in Redis
                await self.redis_client.setex(
                    f"ai_insight:{ai_insight.id}",
                    7200,  # 2 hours TTL
                    json.dumps(asdict(ai_insight), default=str)
                )
                
                return str(ai_insight.id)
                
        except Exception as e:
            logging.error(f"Failed to store AI insight: {e}")
            raise
    
    async def log_audit_event(self, audit_data: Dict[str, Any]):
        """Log audit event"""
        try:
            async with self.get_session() as session:
                audit_log = AuditLog(
                    user_id=audit_data.get('user_id'),
                    action=audit_data.get('action'),
                    resource=audit_data.get('resource'),
                    resource_id=audit_data.get('resource_id'),
                    details=audit_data.get('details', {}),
                    ip_address=audit_data.get('ip_address'),
                    user_agent=audit_data.get('user_agent')
                )
                
                session.add(audit_log)
                
        except Exception as e:
            logging.error(f"Failed to log audit event: {e}")
            raise
    
    async def get_system_metrics(self, metric_name: str, hours: int = 24) -> List[Dict[str, Any]]:
        """Get system metrics"""
        try:
            async with self.get_session() as session:
                cutoff_time = datetime.utcnow() - timedelta(hours=hours)
                
                results = session.query(SystemMetrics).filter(
                    SystemMetrics.metric_name == metric_name,
                    SystemMetrics.timestamp >= cutoff_time
                ).order_by(SystemMetrics.timestamp.desc()).all()
                
                return [asdict(result) for result in results]
                
        except Exception as e:
            logging.error(f"Failed to get system metrics: {e}")
            raise
    
    async def store_system_metric(self, metric_data: Dict[str, Any]):
        """Store system metric"""
        try:
            async with self.get_session() as session:
                system_metric = SystemMetrics(
                    metric_name=metric_data.get('name'),
                    metric_value=metric_data.get('value'),
                    metric_type=metric_data.get('type'),
                    tags=metric_data.get('tags', {})
                )
                
                session.add(system_metric)
                
        except Exception as e:
            logging.error(f"Failed to store system metric: {e}")
            raise
    
    async def execute_query(self, query: str, params: Dict[str, Any] = None) -> List[Dict[str, Any]]:
        """Execute custom SQL query"""
        try:
            async with self.get_session() as session:
                result = session.execute(text(query), params or {})
                columns = result.keys()
                rows = result.fetchall()
                
                return [dict(zip(columns, row)) for row in rows]
                
        except Exception as e:
            logging.error(f"Failed to execute query: {e}")
            raise
    
    async def cleanup_old_data(self, days: int = 30):
        """Cleanup old data based on retention policy"""
        try:
            cutoff_date = datetime.utcnow() - timedelta(days=days)
            
            async with self.get_session() as session:
                # Cleanup old system metrics
                session.execute(
                    text("DELETE FROM system_metrics WHERE timestamp < :cutoff"),
                    {"cutoff": cutoff_date}
                )
                
                # Cleanup old audit logs (keep longer)
                audit_cutoff = datetime.utcnow() - timedelta(days=365)  # 1 year
                session.execute(
                    text("DELETE FROM audit_logs WHERE timestamp < :cutoff"),
                    {"cutoff": audit_cutoff}
                )
                
                # Cleanup old conversations
                conversation_cutoff = datetime.utcnow() - timedelta(days=90)  # 3 months
                session.execute(
                    text("DELETE FROM conversations WHERE timestamp < :cutoff"),
                    {"cutoff": conversation_cutoff}
                )
                
                session.commit()
                
            logging.info(f"Cleaned up data older than {days} days")
            
        except Exception as e:
            logging.error(f"Failed to cleanup old data: {e}")
            raise
    
    async def get_database_stats(self) -> Dict[str, Any]:
        """Get database statistics"""
        try:
            async with self.get_session() as session:
                stats = {}
                
                # Table row counts
                tables = [
                    'intelligence_data', 'user_profiles', 'threat_assessments',
                    'agent_tasks', 'system_metrics', 'audit_logs',
                    'conversations', 'ai_insights'
                ]
                
                for table in tables:
                    result = session.execute(text(f"SELECT COUNT(*) FROM {table}"))
                    stats[f"{table}_count"] = result.scalar()
                
                # Database size
                result = session.execute(text("SELECT pg_size_pretty(pg_database_size(current_database()))"))
                stats['database_size'] = result.scalar()
                
                # Active connections
                result = session.execute(text("SELECT COUNT(*) FROM pg_stat_activity"))
                stats['active_connections'] = result.scalar()
                
                return stats
                
        except Exception as e:
            logging.error(f"Failed to get database stats: {e}")
            raise
    
    async def close(self):
        """Close database connections"""
        try:
            if self.engine:
                self.engine.dispose()
            
            if self.redis_client:
                await self.redis_client.close()
            
            logging.info("Database connections closed")
            
        except Exception as e:
            logging.error(f"Failed to close database connections: {e}")
            raise

# Database migration utilities
class DatabaseMigrator:
    """Handle database migrations"""
    
    def __init__(self, db_manager: DatabaseManager):
        self.db_manager = db_manager
    
    async def run_migrations(self):
        """Run database migrations"""
        try:
            # Create migration tracking table
            async with self.db_manager.get_session() as session:
                session.execute(text("""
                    CREATE TABLE IF NOT EXISTS migrations (
                        id SERIAL PRIMARY KEY,
                        migration_name VARCHAR(255) UNIQUE NOT NULL,
                        applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
                """))
                session.commit()
            
            # Run pending migrations
            await self._apply_migration("001_initial_schema", self._migration_001_initial_schema)
            await self._apply_migration("002_performance_indexes", self._migration_002_performance_indexes)
            await self._apply_migration("003_partitioning", self._migration_003_partitioning)
            
            logging.info("Database migrations completed successfully")
            
        except Exception as e:
            logging.error(f"Migration failed: {e}")
            raise
    
    async def _apply_migration(self, migration_name: str, migration_func):
        """Apply a single migration"""
        try:
            async with self.db_manager.get_session() as session:
                # Check if migration already applied
                result = session.execute(
                    text("SELECT COUNT(*) FROM migrations WHERE migration_name = :name"),
                    {"name": migration_name}
                )
                
                if result.scalar() == 0:
                    await migration_func(session)
                    
                    # Record migration
                    session.execute(
                        text("INSERT INTO migrations (migration_name) VALUES (:name)"),
                        {"name": migration_name}
                    )
                    session.commit()
                    
                    logging.info(f"Applied migration: {migration_name}")
                
        except Exception as e:
            logging.error(f"Failed to apply migration {migration_name}: {e}")
            raise
    
    async def _migration_001_initial_schema(self, session):
        """Initial schema migration"""
        # This is handled by SQLAlchemy Base.metadata.create_all()
        pass
    
    async def _migration_002_performance_indexes(self, session):
        """Performance indexes migration"""
        indexes = [
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_intelligence_data_composite ON intelligence_data (source, classification, timestamp)",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_profiles_composite ON user_profiles (risk_level, behavioral_score, updated_at)",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_threat_assessments_composite ON threat_assessments (threat_level, status, created_at)",
        ]
        
        for index_sql in indexes:
            session.execute(text(index_sql))
    
    async def _migration_003_partitioning(self, session):
        """Table partitioning migration"""
        # Partition intelligence_data by timestamp (monthly)
        session.execute(text("""
            CREATE TABLE IF NOT EXISTS intelligence_data_y2024m01 
            PARTITION OF intelligence_data 
            FOR VALUES FROM ('2024-01-01') TO ('2024-02-01')
        """))
        
        session.execute(text("""
            CREATE TABLE IF NOT EXISTS intelligence_data_y2024m02 
            PARTITION OF intelligence_data 
            FOR VALUES FROM ('2024-02-01') TO ('2024-03-01')
        """))

# Health check utilities
class DatabaseHealthChecker:
    """Database health monitoring"""
    
    def __init__(self, db_manager: DatabaseManager):
        self.db_manager = db_manager
    
    async def check_health(self) -> Dict[str, Any]:
        """Perform comprehensive health check"""
        health_status = {
            "status": "healthy",
            "checks": {},
            "timestamp": datetime.utcnow().isoformat()
        }
        
        try:
            # PostgreSQL health check
            health_status["checks"]["postgresql"] = await self._check_postgresql()
            
            # Redis health check
            health_status["checks"]["redis"] = await self._check_redis()
            
            # Connection pool health
            health_status["checks"]["connection_pool"] = await self._check_connection_pool()
            
            # Overall status
            all_healthy = all(
                check.get("status") == "healthy" 
                for check in health_status["checks"].values()
            )
            
            if not all_healthy:
                health_status["status"] = "unhealthy"
            
        except Exception as e:
            health_status["status"] = "error"
            health_status["error"] = str(e)
        
        return health_status
    
    async def _check_postgresql(self) -> Dict[str, Any]:
        """Check PostgreSQL health"""
        try:
            async with self.db_manager.get_session() as session:
                # Basic connectivity
                result = session.execute(text("SELECT 1"))
                assert result.scalar() == 1
                
                # Check for long-running queries
                result = session.execute(text("""
                    SELECT COUNT(*) FROM pg_stat_activity 
                    WHERE state = 'active' AND query_start < NOW() - INTERVAL '5 minutes'
                """))
                long_queries = result.scalar()
                
                # Check database size
                result = session.execute(text("SELECT pg_database_size(current_database())"))
                db_size = result.scalar()
                
                return {
                    "status": "healthy",
                    "long_running_queries": long_queries,
                    "database_size_bytes": db_size,
                    "database_size_mb": round(db_size / (1024 * 1024), 2)
                }
                
        except Exception as e:
            return {"status": "unhealthy", "error": str(e)}
    
    async def _check_redis(self) -> Dict[str, Any]:
        """Check Redis health"""
        try:
            await self.db_manager.redis_client.ping()
            
            # Check memory usage
            info = await self.db_manager.redis_client.info("memory")
            used_memory = info.get("used_memory", 0)
            max_memory = info.get("maxmemory", 0)
            
            # Check connected clients
            clients_info = await self.db_manager.redis_client.info("clients")
            connected_clients = clients_info.get("connected_clients", 0)
            
            return {
                "status": "healthy",
                "used_memory_bytes": used_memory,
                "used_memory_mb": round(used_memory / (1024 * 1024), 2),
                "connected_clients": connected_clients,
                "memory_usage_percent": round((used_memory / max_memory) * 100, 2) if max_memory > 0 else 0
            }
            
        except Exception as e:
            return {"status": "unhealthy", "error": str(e)}
    
    async def _check_connection_pool(self) -> Dict[str, Any]:
        """Check connection pool health"""
        try:
            pool = self.db_manager.engine.pool
            
            return {
                "status": "healthy",
                "pool_size": pool.size(),
                "checked_in": pool.checkedin(),
                "checked_out": pool.checkedout(),
                "overflow": pool.overflow(),
                "invalid": pool.invalid()
            }
            
        except Exception as e:
            return {"status": "unhealthy", "error": str(e)}
