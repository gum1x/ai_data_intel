-- Initialize Intelligence System Database
-- This script runs when the PostgreSQL container starts for the first time

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create initial database structure
CREATE TABLE IF NOT EXISTS system_info (
    id SERIAL PRIMARY KEY,
    key VARCHAR(100) UNIQUE NOT NULL,
    value TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial system information
INSERT INTO system_info (key, value) VALUES 
    ('database_version', '2.0.0'),
    ('setup_date', CURRENT_TIMESTAMP::TEXT),
    ('status', 'initialized'),
    ('last_migration', '001_initial_schema')
ON CONFLICT (key) DO UPDATE SET 
    value = EXCLUDED.value,
    updated_at = CURRENT_TIMESTAMP;

-- Create migration tracking table
CREATE TABLE IF NOT EXISTS migrations (
    id SERIAL PRIMARY KEY,
    migration_name VARCHAR(255) UNIQUE NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    checksum VARCHAR(64)
);

-- Insert initial migration record
INSERT INTO migrations (migration_name, checksum) VALUES 
    ('001_initial_schema', 'initial')
ON CONFLICT (migration_name) DO NOTHING;

-- Create database statistics view
CREATE OR REPLACE VIEW database_stats AS
SELECT 
    schemaname,
    tablename,
    attname,
    n_distinct,
    correlation,
    most_common_vals,
    most_common_freqs
FROM pg_stats
WHERE schemaname = 'public';

-- Create performance monitoring function
CREATE OR REPLACE FUNCTION get_database_performance()
RETURNS TABLE (
    metric_name TEXT,
    metric_value NUMERIC,
    unit TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        'total_connections'::TEXT,
        (SELECT count(*) FROM pg_stat_activity)::NUMERIC,
        'connections'::TEXT
    UNION ALL
    SELECT 
        'active_connections'::TEXT,
        (SELECT count(*) FROM pg_stat_activity WHERE state = 'active')::NUMERIC,
        'connections'::TEXT
    UNION ALL
    SELECT 
        'database_size_mb'::TEXT,
        (SELECT pg_database_size(current_database()) / 1024 / 1024)::NUMERIC,
        'MB'::TEXT
    UNION ALL
    SELECT 
        'cache_hit_ratio'::TEXT,
        (SELECT round(blks_hit::numeric / (blks_hit + blks_read), 4) FROM pg_stat_database WHERE datname = current_database())::NUMERIC,
        'ratio'::TEXT;
END;
$$ LANGUAGE plpgsql;

-- Create cleanup function for old data
CREATE OR REPLACE FUNCTION cleanup_old_data(retention_days INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
BEGIN
    -- Cleanup old system metrics (if they exist)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'system_metrics') THEN
        DELETE FROM system_metrics WHERE timestamp < CURRENT_TIMESTAMP - INTERVAL '1 day' * retention_days;
        GET DIAGNOSTICS deleted_count = ROW_COUNT;
    END IF;
    
    -- Cleanup old audit logs (keep longer - 1 year)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'audit_logs') THEN
        DELETE FROM audit_logs WHERE timestamp < CURRENT_TIMESTAMP - INTERVAL '1 year';
    END IF;
    
    -- Cleanup old conversations (keep 3 months)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'conversations') THEN
        DELETE FROM conversations WHERE timestamp < CURRENT_TIMESTAMP - INTERVAL '3 months';
    END IF;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create index maintenance function
CREATE OR REPLACE FUNCTION maintain_indexes()
RETURNS TEXT AS $$
DECLARE
    result TEXT := '';
    index_record RECORD;
BEGIN
    -- Reindex all tables
    FOR index_record IN 
        SELECT schemaname, tablename 
        FROM pg_tables 
        WHERE schemaname = 'public'
    LOOP
        EXECUTE format('REINDEX TABLE %I.%I', index_record.schemaname, index_record.tablename);
        result := result || format('Reindexed table: %s.%s\n', index_record.schemaname, index_record.tablename);
    END LOOP;
    
    -- Update table statistics
    ANALYZE;
    result := result || 'Updated table statistics';
    
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Set up automatic cleanup (requires pg_cron extension)
-- Note: This will only work if pg_cron is available
DO $$
BEGIN
    -- Try to create a cleanup job (will fail silently if pg_cron not available)
    BEGIN
        PERFORM cron.schedule('cleanup-old-data', '0 2 * * *', 'SELECT cleanup_old_data(30);');
        INSERT INTO system_info (key, value) VALUES ('pg_cron_enabled', 'true') ON CONFLICT (key) DO UPDATE SET value = 'true';
    EXCEPTION WHEN OTHERS THEN
        INSERT INTO system_info (key, value) VALUES ('pg_cron_enabled', 'false') ON CONFLICT (key) DO UPDATE SET value = 'false';
    END;
END $$;

-- Create database user permissions
GRANT ALL PRIVILEGES ON DATABASE intelligence TO intelligence;
GRANT ALL ON SCHEMA public TO intelligence;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO intelligence;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO intelligence;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO intelligence;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO intelligence;

-- Log completion
INSERT INTO system_info (key, value) VALUES 
    ('initialization_completed', CURRENT_TIMESTAMP::TEXT)
ON CONFLICT (key) DO UPDATE SET 
    value = CURRENT_TIMESTAMP::TEXT,
    updated_at = CURRENT_TIMESTAMP;
