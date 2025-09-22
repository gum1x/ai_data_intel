#!/usr/bin/env python3
"""
Setup script for configuring the Intelligence System with real API keys
"""

import os
import sys
import json
import yaml
from pathlib import Path
from typing import Dict, Any

def create_env_file():
    """Create .env file with API key placeholders"""
    env_content = """# Intelligence System Environment Configuration
# Replace the placeholder values with your actual API keys

# Telegram Bot API Configuration
TELEGRAM_BOT_TOKEN=your_telegram_bot_token_here
TELEGRAM_API_ID=your_telegram_api_id_here
TELEGRAM_API_HASH=your_telegram_api_hash_here
TELEGRAM_PHONE_NUMBER=+1234567890

# Database Configuration
DATABASE_URL=postgresql://intelligence:password@localhost:5432/intelligence
REDIS_URL=redis://localhost:6379

# Kafka Configuration
KAFKA_BOOTSTRAP_SERVERS=localhost:9092

# Security Configuration
JWT_SECRET=your-super-secret-jwt-key-here-must-be-32-chars-min
ENCRYPTION_KEY=your-super-secret-encryption-key-32-chars

# AI Provider API Keys
OPENAI_API_KEY=your-openai-api-key-here
ANTHROPIC_API_KEY=your-anthropic-api-key-here
GOOGLE_API_KEY=your-google-api-key-here
HUGGINGFACE_API_KEY=your-huggingface-api-key-here
COHERE_API_KEY=your-cohere-api-key-here
TOGETHER_API_KEY=your-together-api-key-here

# Monitoring Configuration
PROMETHEUS_ENDPOINT=http://localhost:9090
JAEGER_ENDPOINT=http://localhost:14268/api/traces

# System Configuration
LOG_LEVEL=INFO
ENVIRONMENT=development
"""
    
    with open('.env', 'w') as f:
        f.write(env_content)
    
    print("✅ Created .env file with API key placeholders")
    print("📝 Please edit .env file and add your actual API keys")

def create_config_yaml():
    """Create config.yaml from template"""
    config_path = Path("rust-intelligence-system/config.yaml")
    template_path = Path("rust-intelligence-system/config.template.yaml")
    
    if template_path.exists() and not config_path.exists():
        with open(template_path, 'r') as f:
            config_content = f.read()
        
        with open(config_path, 'w') as f:
            f.write(config_content)
        
        print("✅ Created config.yaml from template")
        print("📝 Please edit config.yaml and add your actual API keys")
    else:
        print("⚠️  config.yaml already exists or template not found")

def create_telegram_bot_instructions():
    """Create instructions for setting up Telegram Client API"""
    instructions = """
# 🤖 Telegram Client API Setup Instructions

## ⚠️ IMPORTANT: Use Client API, Not Bot API
The system now uses the Telegram Client API (MTProto) instead of the Bot API because:
- ✅ Can join private channels and groups
- ✅ Can access full user information
- ✅ Can read message history
- ✅ Can get detailed chat information
- ❌ Bot API is limited and can't access private channels

## Step 1: Get Telegram API Credentials
1. Go to https://my.telegram.org/auth
2. Log in with your phone number
3. Go to "API development tools"
4. Create a new application:
   - App title: "Intelligence System"
   - Short name: "intel_sys"
   - Platform: "Desktop"
   - Description: "AI Intelligence System for data collection"
5. Copy the API ID and API Hash

## Step 2: Configure the System
1. Edit the .env file and add:
   - TELEGRAM_API_ID=your_api_id_from_my_telegram_org
   - TELEGRAM_API_HASH=your_api_hash_from_my_telegram_org
   - TELEGRAM_PHONE_NUMBER=your_phone_number_with_country_code
   - TELEGRAM_SESSION_STRING=your_session_string_here (optional)

## Step 3: Authentication Process
1. Start the system: `python run_system.py`
2. The system will prompt for phone number verification
3. Enter the verification code sent to your phone
4. If you have 2FA enabled, enter your password
5. The system will save the session for future use

## Step 4: Join Target Channels
1. Manually join the channels/groups you want to monitor
2. The system will automatically start collecting data
3. Check the logs to see collection progress

## Important Notes:
- ⚠️ This uses YOUR Telegram account - be careful with what you access
- 🔒 Keep your API credentials secure and never commit them to version control
- 📱 The phone number must be the same as your Telegram account
- 🚫 Don't abuse the API - respect rate limits and Telegram's ToS
- 💾 Session strings allow reconnection without re-authentication
- 🔄 Rate limits: 30 requests per second (configurable)

## Security Considerations:
- Use a dedicated Telegram account for the system
- Don't use your personal account
- Monitor API usage to avoid account restrictions
- Keep session strings secure (they provide full account access)
- Consider using Telegram's test servers for development

## Troubleshooting:
- If authentication fails, check your phone number format (+country_code)
- If you get "PHONE_NUMBER_INVALID", ensure the number matches your Telegram account
- If you get rate limited, reduce the rate_limit_per_second setting
- If session expires, delete the session string and re-authenticate
"""
    
    with open("TELEGRAM_SETUP.md", 'w') as f:
        f.write(instructions)
    
    print("✅ Created TELEGRAM_SETUP.md with detailed instructions")

def create_ai_provider_instructions():
    """Create instructions for setting up AI providers"""
    instructions = """
# 🤖 AI Provider Setup Instructions

## OpenAI Setup
1. Go to https://platform.openai.com/api-keys
2. Create a new API key
3. Add to .env: OPENAI_API_KEY=your_openai_api_key

## Anthropic Setup
1. Go to https://console.anthropic.com/
2. Create an account and get API key
3. Add to .env: ANTHROPIC_API_KEY=your_anthropic_api_key

## Google AI Setup
1. Go to https://aistudio.google.com/app/apikey
2. Create a new API key
3. Add to .env: GOOGLE_API_KEY=your_google_api_key

## Hugging Face Setup
1. Go to https://huggingface.co/settings/tokens
2. Create a new access token
3. Add to .env: HUGGINGFACE_API_KEY=your_huggingface_token

## Cohere Setup
1. Go to https://dashboard.cohere.ai/api-keys
2. Create a new API key
3. Add to .env: COHERE_API_KEY=your_cohere_api_key

## Together AI Setup
1. Go to https://api.together.xyz/settings/api-keys
2. Create a new API key
3. Add to .env: TOGETHER_API_KEY=your_together_api_key

## Important Notes:
- All API keys have usage limits and costs
- Monitor your usage to avoid unexpected charges
- Keep API keys secure and never share them publicly
- Some providers offer free tiers for testing
"""
    
    with open("AI_PROVIDERS_SETUP.md", 'w') as f:
        f.write(instructions)
    
    print("✅ Created AI_PROVIDERS_SETUP.md with setup instructions")

def create_database_setup_instructions():
    """Create instructions for setting up databases"""
    instructions = """
# 🗄️ Database Setup Instructions

## PostgreSQL Setup
1. Install PostgreSQL:
   - Ubuntu/Debian: `sudo apt install postgresql postgresql-contrib`
   - macOS: `brew install postgresql`
   - Windows: Download from https://www.postgresql.org/download/

2. Create database and user:
   ```sql
   sudo -u postgres psql
   CREATE DATABASE intelligence;
   CREATE USER intelligence WITH PASSWORD 'password';
   GRANT ALL PRIVILEGES ON DATABASE intelligence TO intelligence;
   \q
   ```

3. Update .env:
   ```
   DATABASE_URL=postgresql://intelligence:password@localhost:5432/intelligence
   ```

## Redis Setup
1. Install Redis:
   - Ubuntu/Debian: `sudo apt install redis-server`
   - macOS: `brew install redis`
   - Windows: Download from https://redis.io/download

2. Start Redis:
   ```bash
   redis-server
   ```

3. Update .env:
   ```
   REDIS_URL=redis://localhost:6379
   ```

## Kafka Setup
1. Download Kafka from https://kafka.apache.org/downloads
2. Extract and start Zookeeper:
   ```bash
   bin/zookeeper-server-start.sh config/zookeeper.properties
   ```
3. Start Kafka:
   ```bash
   bin/kafka-server-start.sh config/server.properties
   ```

4. Update .env:
   ```
   KAFKA_BOOTSTRAP_SERVERS=localhost:9092
   ```

## Docker Alternative
Use Docker Compose for easier setup:
```bash
cd rust-intelligence-system
docker-compose up -d postgres redis kafka
```
"""
    
    with open("DATABASE_SETUP.md", 'w') as f:
        f.write(instructions)
    
    print("✅ Created DATABASE_SETUP.md with database setup instructions")

def create_security_setup_instructions():
    """Create instructions for security configuration"""
    instructions = """
# 🔒 Security Setup Instructions

## Generate Secure Keys
1. Generate JWT Secret (32+ characters):
   ```bash
   openssl rand -base64 32
   ```

2. Generate Encryption Key (32 characters):
   ```bash
   openssl rand -hex 16
   ```

3. Update .env:
   ```
   JWT_SECRET=your_generated_jwt_secret_here
   ENCRYPTION_KEY=your_generated_encryption_key_here
   ```

## Security Best Practices
1. **Never commit API keys to version control**
2. **Use environment variables for all secrets**
3. **Enable HTTPS in production**
4. **Use strong passwords for database users**
5. **Regularly rotate API keys and secrets**
6. **Monitor API usage for anomalies**
7. **Use firewall rules to restrict access**
8. **Enable audit logging**

## Production Security Checklist
- [ ] All API keys are in environment variables
- [ ] Database passwords are strong and unique
- [ ] JWT secrets are cryptographically secure
- [ ] HTTPS is enabled for all endpoints
- [ ] Rate limiting is configured
- [ ] Audit logging is enabled
- [ ] Firewall rules are in place
- [ ] Regular security updates are applied
"""
    
    with open("SECURITY_SETUP.md", 'w') as f:
        f.write(instructions)
    
    print("✅ Created SECURITY_SETUP.md with security instructions")

def main():
    """Main setup function"""
    print("🚀 Intelligence System Real API Setup")
    print("=" * 50)
    
    # Create configuration files
    create_env_file()
    create_config_yaml()
    
    # Create setup instructions
    create_telegram_bot_instructions()
    create_ai_provider_instructions()
    create_database_setup_instructions()
    create_security_setup_instructions()
    
    print("\n" + "=" * 50)
    print("✅ Setup complete!")
    print("\n📋 Next steps:")
    print("1. Edit .env file with your actual API keys")
    print("2. Follow the setup instructions in the generated .md files")
    print("3. Start the system: python run_system.py")
    print("\n📚 Documentation created:")
    print("- TELEGRAM_SETUP.md")
    print("- AI_PROVIDERS_SETUP.md") 
    print("- DATABASE_SETUP.md")
    print("- SECURITY_SETUP.md")
    print("\n⚠️  Remember to keep your API keys secure!")

if __name__ == "__main__":
    main()
