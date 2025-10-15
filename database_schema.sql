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

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for updated_at
CREATE TRIGGER update_messages_updated_at BEFORE UPDATE ON messages FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_chat_analysis_updated_at BEFORE UPDATE ON chat_analysis FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_patterns_updated_at BEFORE UPDATE ON user_patterns FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_telegram_status_updated_at BEFORE UPDATE ON telegram_status FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert initial telegram status
INSERT INTO telegram_status (is_connected, last_connection, messages_processed, chats_monitored) 
VALUES (FALSE, NULL, 0, 0) 
ON CONFLICT DO NOTHING;

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
