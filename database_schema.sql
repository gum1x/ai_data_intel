-- AI Intelligence System Database Schema
-- This will be executed in Supabase SQL Editor

-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Messages table to store all Telegram messages
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id BIGINT NOT NULL,
    chat_id BIGINT NOT NULL,
    user_id BIGINT,
    username VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    message_text TEXT,
    message_date TIMESTAMP WITH TIME ZONE NOT NULL,
    message_type VARCHAR(50) DEFAULT 'text',
    reply_to_message_id BIGINT,
    is_bot BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Chat analysis table for storing learned patterns
CREATE TABLE IF NOT EXISTS chat_analysis (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_id BIGINT NOT NULL UNIQUE,
    chat_title VARCHAR(255),
    chat_type VARCHAR(50),
    total_messages INTEGER DEFAULT 0,
    analyzed_messages INTEGER DEFAULT 0,
    common_words JSONB,
    average_message_length DECIMAL(10,2),
    emoji_usage_ratio DECIMAL(5,4),
    punctuation_patterns JSONB,
    common_phrases JSONB,
    response_time_patterns JSONB,
    active_users JSONB,
    last_message_date TIMESTAMP WITH TIME ZONE,
    last_analyzed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User behavior patterns
CREATE TABLE IF NOT EXISTS user_patterns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    username VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    chat_id BIGINT NOT NULL,
    message_count INTEGER DEFAULT 0,
    average_response_time DECIMAL(10,2),
    common_words JSONB,
    common_phrases JSONB,
    emoji_preferences JSONB,
    active_hours JSONB,
    last_seen TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, chat_id)
);

-- Auto responses log
CREATE TABLE IF NOT EXISTS auto_responses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_id BIGINT NOT NULL,
    original_message_id BIGINT,
    response_text TEXT NOT NULL,
    response_type VARCHAR(50) DEFAULT 'style_based',
    confidence_score DECIMAL(5,4),
    user_id BIGINT,
    sent_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- System logs
CREATE TABLE IF NOT EXISTS system_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    log_level VARCHAR(20) NOT NULL,
    component VARCHAR(100) NOT NULL,
    message TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Telegram connection status
CREATE TABLE IF NOT EXISTS telegram_status (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_connected BOOLEAN DEFAULT FALSE,
    last_connection TIMESTAMP WITH TIME ZONE,
    connection_error TEXT,
    messages_processed INTEGER DEFAULT 0,
    chats_monitored INTEGER DEFAULT 0,
    last_activity TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Person Intelligence Tables
CREATE TABLE IF NOT EXISTS person_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    username VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    phone VARCHAR(50),
    email VARCHAR(255),
    bio TEXT,
    location VARCHAR(255),
    timezone VARCHAR(50),
    language VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE TABLE IF NOT EXISTS person_behavior_patterns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    pattern_type VARCHAR(100) NOT NULL, -- 'communication_style', 'response_time', 'activity_hours', 'topic_preferences'
    pattern_data JSONB NOT NULL,
    confidence_score DECIMAL(3,2) DEFAULT 0.0,
    first_observed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_observed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    observation_count INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS person_intelligence (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    intelligence_type VARCHAR(100) NOT NULL, -- 'personal_info', 'social_connections', 'interests', 'goals', 'vulnerabilities'
    intelligence_data JSONB NOT NULL,
    source_message_id UUID REFERENCES messages(id),
    confidence_level DECIMAL(3,2) DEFAULT 0.0,
    extraction_method VARCHAR(100), -- 'ai_analysis', 'pattern_recognition', 'social_engineering'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    verified_at TIMESTAMP WITH TIME ZONE,
    verification_status VARCHAR(50) DEFAULT 'unverified' -- 'verified', 'unverified', 'disputed'
);

CREATE TABLE IF NOT EXISTS person_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    related_user_id BIGINT NOT NULL,
    relationship_type VARCHAR(100) NOT NULL, -- 'friend', 'colleague', 'family', 'romantic', 'business'
    relationship_strength DECIMAL(3,2) DEFAULT 0.0,
    interaction_frequency INTEGER DEFAULT 0,
    last_interaction TIMESTAMP WITH TIME ZONE,
    relationship_context TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS person_activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    activity_type VARCHAR(100) NOT NULL, -- 'message_sent', 'login', 'profile_update', 'location_change'
    activity_data JSONB,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    source_chat_id BIGINT,
    metadata JSONB
);

-- Chat monitoring configuration
CREATE TABLE IF NOT EXISTS monitored_chats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_id BIGINT NOT NULL UNIQUE,
    chat_title VARCHAR(255),
    chat_type VARCHAR(50), -- 'group', 'supergroup', 'channel', 'private'
    is_active BOOLEAN DEFAULT TRUE,
    auto_analyze BOOLEAN DEFAULT TRUE,
    auto_respond BOOLEAN DEFAULT FALSE,
    analysis_frequency INTEGER DEFAULT 60, -- seconds between analysis cycles
    last_analyzed TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Message analysis tracking
CREATE TABLE IF NOT EXISTS message_analysis_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID REFERENCES messages(id),
    analysis_type VARCHAR(100) NOT NULL, -- 'message_understanding', 'person_intelligence', 'chat_pattern'
    analysis_status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'completed', 'failed'
    analysis_data JSONB,
    error_message TEXT,
    processing_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_user_id ON messages(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(message_date);
CREATE INDEX IF NOT EXISTS idx_messages_text ON messages USING gin(to_tsvector('english', message_text));

CREATE INDEX IF NOT EXISTS idx_chat_analysis_chat_id ON chat_analysis(chat_id);
CREATE INDEX IF NOT EXISTS idx_user_patterns_user_chat ON user_patterns(user_id, chat_id);
CREATE INDEX IF NOT EXISTS idx_auto_responses_chat_id ON auto_responses(chat_id);
CREATE INDEX IF NOT EXISTS idx_system_logs_level ON system_logs(log_level);
CREATE INDEX IF NOT EXISTS idx_system_logs_component ON system_logs(component);

-- Person intelligence indexes
CREATE INDEX IF NOT EXISTS idx_person_profiles_user_id ON person_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_person_behavior_user_id ON person_behavior_patterns(user_id);
CREATE INDEX IF NOT EXISTS idx_person_intelligence_user_id ON person_intelligence(user_id);
CREATE INDEX IF NOT EXISTS idx_person_intelligence_type ON person_intelligence(intelligence_type);
CREATE INDEX IF NOT EXISTS idx_person_relationships_user_id ON person_relationships(user_id);
CREATE INDEX IF NOT EXISTS idx_person_relationships_related ON person_relationships(related_user_id);
CREATE INDEX IF NOT EXISTS idx_person_activities_user_id ON person_activities(user_id);
CREATE INDEX IF NOT EXISTS idx_person_activities_timestamp ON person_activities(timestamp);

-- Monitoring and analysis indexes
CREATE INDEX IF NOT EXISTS idx_monitored_chats_chat_id ON monitored_chats(chat_id);
CREATE INDEX IF NOT EXISTS idx_monitored_chats_active ON monitored_chats(is_active);
CREATE INDEX IF NOT EXISTS idx_message_analysis_log_message_id ON message_analysis_log(message_id);
CREATE INDEX IF NOT EXISTS idx_message_analysis_log_status ON message_analysis_log(analysis_status);
CREATE INDEX IF NOT EXISTS idx_message_analysis_log_type ON message_analysis_log(analysis_type);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for updated_at
DROP TRIGGER IF EXISTS update_messages_updated_at ON messages;
CREATE TRIGGER update_messages_updated_at BEFORE UPDATE ON messages FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_chat_analysis_updated_at ON chat_analysis;
CREATE TRIGGER update_chat_analysis_updated_at BEFORE UPDATE ON chat_analysis FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_user_patterns_updated_at ON user_patterns;
CREATE TRIGGER update_user_patterns_updated_at BEFORE UPDATE ON user_patterns FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_telegram_status_updated_at ON telegram_status;
CREATE TRIGGER update_telegram_status_updated_at BEFORE UPDATE ON telegram_status FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_person_profiles_updated_at ON person_profiles;
CREATE TRIGGER update_person_profiles_updated_at BEFORE UPDATE ON person_profiles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_person_relationships_updated_at ON person_relationships;
CREATE TRIGGER update_person_relationships_updated_at BEFORE UPDATE ON person_relationships FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_monitored_chats_updated_at ON monitored_chats;
CREATE TRIGGER update_monitored_chats_updated_at BEFORE UPDATE ON monitored_chats FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert initial telegram status (only if table is empty)
INSERT INTO telegram_status (is_connected, last_connection, messages_processed, chats_monitored) 
SELECT FALSE, NULL, 0, 0
WHERE NOT EXISTS (SELECT 1 FROM telegram_status);

-- Create a view for active chats summary
CREATE OR REPLACE VIEW active_chats_summary AS
SELECT 
    ca.chat_id,
    ca.chat_title,
    ca.chat_type,
    ca.total_messages,
    ca.analyzed_messages,
    ca.last_message_date,
    ca.last_analyzed,
    COUNT(DISTINCT m.user_id) as unique_users,
    MAX(m.message_date) as latest_message
FROM chat_analysis ca
LEFT JOIN messages m ON ca.chat_id = m.chat_id
GROUP BY ca.chat_id, ca.chat_title, ca.chat_type, ca.total_messages, 
         ca.analyzed_messages, ca.last_message_date, ca.last_analyzed;

-- Create a view for user activity summary
CREATE OR REPLACE VIEW user_activity_summary AS
SELECT 
    up.user_id,
    up.username,
    up.first_name,
    up.last_name,
    COUNT(DISTINCT up.chat_id) as active_chats,
    SUM(up.message_count) as total_messages,
    AVG(up.average_response_time) as avg_response_time,
    MAX(up.last_seen) as last_seen
FROM user_patterns up
GROUP BY up.user_id, up.username, up.first_name, up.last_name;

-- Create a view for person intelligence summary
CREATE OR REPLACE VIEW person_intelligence_summary AS
SELECT 
    pp.user_id,
    pp.username,
    pp.first_name,
    pp.last_name,
    pp.location,
    pp.language,
    COUNT(DISTINCT pi.id) as intelligence_count,
    COUNT(DISTINCT pb.id) as behavior_patterns_count,
    COUNT(DISTINCT pr.id) as relationships_count,
    COUNT(DISTINCT pa.id) as activities_count,
    MAX(pi.created_at) as last_intelligence_update,
    MAX(pa.timestamp) as last_activity
FROM person_profiles pp
LEFT JOIN person_intelligence pi ON pp.user_id = pi.user_id
LEFT JOIN person_behavior_patterns pb ON pp.user_id = pb.user_id
LEFT JOIN person_relationships pr ON pp.user_id = pr.user_id
LEFT JOIN person_activities pa ON pp.user_id = pa.user_id
GROUP BY pp.user_id, pp.username, pp.first_name, pp.last_name, pp.location, pp.language;

-- Create a view for monitoring status
CREATE OR REPLACE VIEW monitoring_status AS
SELECT 
    mc.chat_id,
    mc.chat_title,
    mc.chat_type,
    mc.is_active,
    mc.auto_analyze,
    mc.auto_respond,
    mc.last_analyzed,
    COUNT(DISTINCT m.id) as total_messages,
    COUNT(DISTINCT CASE WHEN m.message_date > mc.last_analyzed THEN m.id END) as unanalyzed_messages,
    COUNT(DISTINCT mal.id) as pending_analyses
FROM monitored_chats mc
LEFT JOIN messages m ON mc.chat_id = m.chat_id
LEFT JOIN message_analysis_log mal ON m.id = mal.message_id AND mal.analysis_status = 'pending'
GROUP BY mc.chat_id, mc.chat_title, mc.chat_type, mc.is_active, mc.auto_analyze, mc.auto_respond, mc.last_analyzed;
