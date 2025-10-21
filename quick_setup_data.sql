-- QUICK SUPABASE SETUP & DATA INSERTION SCRIPT
-- Run this if the verification script shows missing data

-- 1. Ensure telegram_status has initial data
INSERT INTO telegram_status (is_connected, last_connection, messages_processed, chats_monitored) 
SELECT FALSE, NULL, 0, 0
WHERE NOT EXISTS (SELECT 1 FROM telegram_status);

-- 2. Ensure monitored_chats has the test chat
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

-- 3. Insert test messages if none exist
INSERT INTO messages (message_id, chat_id, user_id, username, first_name, last_name, message_text, message_date, message_type, is_bot) 
SELECT 1, -1001234567890, 12345, 'john_doe', 'John', 'Doe', 'Hey everyone! How is the project going? I am working on the AI features and need some help with the database connection.', NOW() - INTERVAL '1 hour', 'text', false
WHERE NOT EXISTS (SELECT 1 FROM messages WHERE chat_id = -1001234567890);

INSERT INTO messages (message_id, chat_id, user_id, username, first_name, last_name, message_text, message_date, message_type, is_bot) 
SELECT 2, -1001234567890, 67890, 'jane_smith', 'Jane', 'Smith', 'Great progress! I just finished implementing the message analysis. My email is jane@company.com if you need to reach me.', NOW() - INTERVAL '50 minutes', 'text', false
WHERE NOT EXISTS (SELECT 1 FROM messages WHERE message_id = 2 AND chat_id = -1001234567890);

INSERT INTO messages (message_id, chat_id, user_id, username, first_name, last_name, message_text, message_date, message_type, is_bot) 
SELECT 3, -1001234567890, 11111, 'mike_wilson', 'Mike', 'Wilson', 'That is awesome! I am struggling with some personal issues lately, but the project keeps me motivated. My phone is +1-555-123-4567.', NOW() - INTERVAL '40 minutes', 'text', false
WHERE NOT EXISTS (SELECT 1 FROM messages WHERE message_id = 3 AND chat_id = -1001234567890);

INSERT INTO messages (message_id, chat_id, user_id, username, first_name, last_name, message_text, message_date, message_type, is_bot) 
SELECT 4, -1001234567890, 22222, 'sarah_jones', 'Sarah', 'Jones', 'I am so excited about this! My goal is to become a senior developer. I live in San Francisco and love hiking on weekends.', NOW() - INTERVAL '30 minutes', 'text', false
WHERE NOT EXISTS (SELECT 1 FROM messages WHERE message_id = 4 AND chat_id = -1001234567890);

INSERT INTO messages (message_id, chat_id, user_id, username, first_name, last_name, message_text, message_date, message_type, is_bot) 
SELECT 5, -1001234567890, 33333, 'alex_brown', 'Alex', 'Brown', 'This is incredible work! I have been dealing with some financial stress, but this project gives me hope for the future.', NOW() - INTERVAL '20 minutes', 'text', false
WHERE NOT EXISTS (SELECT 1 FROM messages WHERE message_id = 5 AND chat_id = -1001234567890);

-- 4. Insert chat analysis if none exists
INSERT INTO chat_analysis (chat_id, chat_title, chat_type, total_messages, analyzed_messages, last_message_date) 
SELECT -1001234567890, 'AI Development Team', 'group', 
       (SELECT COUNT(*) FROM messages WHERE chat_id = -1001234567890), 
       0, 
       (SELECT MAX(message_date) FROM messages WHERE chat_id = -1001234567890)
WHERE NOT EXISTS (SELECT 1 FROM chat_analysis WHERE chat_id = -1001234567890);

-- 5. Update telegram status with current counts
UPDATE telegram_status SET 
    messages_processed = (SELECT COUNT(*) FROM messages),
    chats_monitored = (SELECT COUNT(*) FROM monitored_chats WHERE is_active = true),
    last_activity = NOW()
WHERE id = (SELECT id FROM telegram_status LIMIT 1);

-- 6. Show final status
SELECT 'SETUP COMPLETE' as status;
SELECT 'Messages:' as table_name, COUNT(*) as count FROM messages
UNION ALL
SELECT 'Monitored Chats:', COUNT(*) FROM monitored_chats WHERE is_active = true
UNION ALL
SELECT 'Telegram Status:', COUNT(*) FROM telegram_status
UNION ALL
SELECT 'Chat Analysis:', COUNT(*) FROM chat_analysis;
