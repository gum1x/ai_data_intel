-- Insert test messages to demonstrate real message reading
-- Run this in Supabase SQL Editor after applying the main schema

INSERT INTO messages (message_id, chat_id, user_id, username, first_name, last_name, message_text, message_date, message_type, is_bot) VALUES
(1, -1001234567890, 12345, 'john_doe', 'John', 'Doe', 'Hey everyone! How is the project going? I am working on the AI features and need some help with the database connection.', NOW() - INTERVAL '1 hour', 'text', false),
(2, -1001234567890, 67890, 'jane_smith', 'Jane', 'Smith', 'Great progress! I just finished implementing the message analysis. My email is jane@company.com if you need to reach me.', NOW() - INTERVAL '50 minutes', 'text', false),
(3, -1001234567890, 11111, 'mike_wilson', 'Mike', 'Wilson', 'That is awesome! I am struggling with some personal issues lately, but the project keeps me motivated. My phone is +1-555-123-4567.', NOW() - INTERVAL '40 minutes', 'text', false),
(4, -1001234567890, 22222, 'sarah_jones', 'Sarah', 'Jones', 'I am so excited about this! My goal is to become a senior developer. I live in San Francisco and love hiking on weekends.', NOW() - INTERVAL '30 minutes', 'text', false),
(5, -1001234567890, 33333, 'alex_brown', 'Alex', 'Brown', 'This is incredible work! I have been dealing with some financial stress, but this project gives me hope for the future.', NOW() - INTERVAL '20 minutes', 'text', false),
(6, -1001234567890, 44444, 'emma_davis', 'Emma', 'Davis', 'I am really proud of what we have built together. My dream is to start my own tech company someday. I am currently in New York.', NOW() - INTERVAL '10 minutes', 'text', false),
(7, -1001234567890, 55555, 'david_miller', 'David', 'Miller', 'Amazing! I have been learning so much from this project. My family is really supportive of my career change into tech.', NOW() - INTERVAL '5 minutes', 'text', false),
(8, -1001234567890, 66666, 'lisa_garcia', 'Lisa', 'Garcia', 'This is fantastic! I am working on improving my coding skills. I have been dealing with some anxiety about job interviews.', NOW() - INTERVAL '2 minutes', 'text', false),
(9, -1001234567890, 77777, 'tom_white', 'Tom', 'White', 'Excellent work everyone! I am really excited about the future of AI. My goal is to make technology more accessible to everyone.', NOW() - INTERVAL '1 minute', 'text', false),
(10, -1001234567890, 88888, 'anna_taylor', 'Anna', 'Taylor', 'This is incredible! I am so grateful to be part of this team. I am working on my master degree in computer science at Stanford.', NOW(), 'text', false);

-- Insert some chat analysis data
INSERT INTO chat_analysis (chat_id, chat_title, chat_type, total_messages, analyzed_messages, last_message_date) VALUES
(-1001234567890, 'AI Development Team', 'group', 10, 0, NOW());

-- Update telegram status
UPDATE telegram_status SET 
    is_connected = true,
    last_connection = NOW(),
    messages_processed = 10,
    chats_monitored = 1,
    last_activity = NOW();
