-- Add monitored chat configuration
-- Run this after applying the main database schema

-- Add the test chat to monitored chats
INSERT INTO monitored_chats (chat_id, chat_title, chat_type, is_active, auto_analyze, auto_respond, analysis_frequency) 
VALUES (-1001234567890, 'AI Development Team', 'group', true, true, false, 60)
ON CONFLICT (chat_id) DO UPDATE SET
    chat_title = EXCLUDED.chat_title,
    chat_type = EXCLUDED.chat_type,
    is_active = EXCLUDED.is_active,
    auto_analyze = EXCLUDED.auto_analyze,
    auto_respond = EXCLUDED.auto_respond,
    analysis_frequency = EXCLUDED.analysis_frequency,
    updated_at = NOW();

-- You can add more chats like this:
-- INSERT INTO monitored_chats (chat_id, chat_title, chat_type, is_active, auto_analyze, auto_respond, analysis_frequency) 
-- VALUES (-1001234567891, 'Another Chat', 'group', true, true, false, 120)
-- ON CONFLICT (chat_id) DO UPDATE SET
--     chat_title = EXCLUDED.chat_title,
--     chat_type = EXCLUDED.chat_type,
--     is_active = EXCLUDED.is_active,
--     auto_analyze = EXCLUDED.auto_analyze,
--     auto_respond = EXCLUDED.auto_respond,
--     analysis_frequency = EXCLUDED.analysis_frequency,
--     updated_at = NOW();

-- Check monitoring status
SELECT * FROM monitoring_status;
