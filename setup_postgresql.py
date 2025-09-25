#!/usr/bin/env python3
"""
PostgreSQL Database Setup Script
Creates database, user, and initializes schema for the Intelligence System
"""

import asyncio
import asyncpg
import os
import sys
from pathlib import Path

# Database configuration
DB_HOST = os.getenv('POSTGRES_HOST', 'localhost')
DB_PORT = int(os.getenv('POSTGRES_PORT', 5432))
DB_NAME = os.getenv('POSTGRES_DB', 'intelligence')
DB_USER = os.getenv('POSTGRES_USER', 'intelligence')
DB_PASSWORD = os.getenv('POSTGRES_PASSWORD', 'intelligence_password')
ADMIN_USER = os.getenv('POSTGRES_ADMIN_USER', 'postgres')
ADMIN_PASSWORD = os.getenv('POSTGRES_ADMIN_PASSWORD', 'postgres')

async def create_database_and_user():
    """Create database and user with proper permissions"""
    try:
        # Connect as admin user to create database and user
        admin_conn = await asyncpg.connect(
            host=DB_HOST,
            port=DB_PORT,
            user=ADMIN_USER,
            password=ADMIN_PASSWORD,
            database='postgres'
        )
        
        print(f"Connected to PostgreSQL as admin user: {ADMIN_USER}")
        
        # Create user if it doesn't exist
        try:
            await admin_conn.execute(f"""
                CREATE USER {DB_USER} WITH PASSWORD '{DB_PASSWORD}';
            """)
            print(f"Created user: {DB_USER}")
        except asyncpg.DuplicateObjectError:
            print(f"User {DB_USER} already exists")
        
        # Create database if it doesn't exist
        try:
            await admin_conn.execute(f"""
                CREATE DATABASE {DB_NAME} OWNER {DB_USER};
            """)
            print(f"Created database: {DB_NAME}")
        except asyncpg.DuplicateDatabaseError:
            print(f"Database {DB_NAME} already exists")
        
        # Grant permissions
        await admin_conn.execute(f"""
            GRANT ALL PRIVILEGES ON DATABASE {DB_NAME} TO {DB_USER};
        """)
        
        # Grant schema permissions
        await admin_conn.execute(f"""
            GRANT ALL ON SCHEMA public TO {DB_USER};
        """)
        
        # Grant table permissions (for future tables)
        await admin_conn.execute(f"""
            GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO {DB_USER};
            GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO {DB_USER};
        """)
        
        # Set default privileges for future tables
        await admin_conn.execute(f"""
            ALTER DEFAULT PRIVILEGES IN SCHEMA public 
            GRANT ALL ON TABLES TO {DB_USER};
            ALTER DEFAULT PRIVILEGES IN SCHEMA public 
            GRANT ALL ON SEQUENCES TO {DB_USER};
        """)
        
        print("Database permissions configured successfully")
        
        await admin_conn.close()
        
    except Exception as e:
        print(f"Error creating database and user: {e}")
        sys.exit(1)

async def test_connection():
    """Test connection to the new database"""
    try:
        conn = await asyncpg.connect(
            host=DB_HOST,
            port=DB_PORT,
            user=DB_USER,
            password=DB_PASSWORD,
            database=DB_NAME
        )
        
        # Test basic query
        result = await conn.fetchval("SELECT version()")
        print(f"Connection successful! PostgreSQL version: {result}")
        
        # Test creating a simple table
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS test_table (
                id SERIAL PRIMARY KEY,
                test_data TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)
        
        # Insert test data
        await conn.execute("""
            INSERT INTO test_table (test_data) VALUES ('Database setup test')
        """)
        
        # Verify data
        count = await conn.fetchval("SELECT COUNT(*) FROM test_table")
        print(f"Test table created and data inserted. Row count: {count}")
        
        # Clean up test table
        await conn.execute("DROP TABLE IF EXISTS test_table")
        
        await conn.close()
        print("Database connection test completed successfully")
        
    except Exception as e:
        print(f"Error testing database connection: {e}")
        sys.exit(1)

async def create_initial_schema():
    """Create initial database schema"""
    try:
        conn = await asyncpg.connect(
            host=DB_HOST,
            port=DB_PORT,
            user=DB_USER,
            password=DB_PASSWORD,
            database=DB_NAME
        )
        
        print("Creating initial database schema...")
        
        # Create extensions
        await conn.execute("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
        await conn.execute("CREATE EXTENSION IF NOT EXISTS \"pg_trgm\";")
        await conn.execute("CREATE EXTENSION IF NOT EXISTS \"btree_gin\";")
        
        # Create initial tables (basic structure)
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS system_info (
                id SERIAL PRIMARY KEY,
                key VARCHAR(100) UNIQUE NOT NULL,
                value TEXT,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)
        
        # Insert initial system info
        await conn.execute("""
            INSERT INTO system_info (key, value) VALUES 
            ('database_version', '2.0.0'),
            ('setup_date', CURRENT_TIMESTAMP::TEXT),
            ('status', 'initialized')
            ON CONFLICT (key) DO UPDATE SET 
            value = EXCLUDED.value,
            updated_at = CURRENT_TIMESTAMP
        """)
        
        print("Initial schema created successfully")
        
        await conn.close()
        
    except Exception as e:
        print(f"Error creating initial schema: {e}")
        sys.exit(1)

async def setup_environment_file():
    """Create or update environment file with database configuration"""
    env_file = Path('.env')
    
    env_content = f"""# Database Configuration
DATABASE_HOST={DB_HOST}
DATABASE_PORT={DB_PORT}
DATABASE_NAME={DB_NAME}
DATABASE_USER={DB_USER}
DATABASE_PASSWORD={DB_PASSWORD}

# Redis Configuration
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DB=0

# Security Configuration
JWT_SECRET=your-super-secret-jwt-key-here-must-be-32-chars-min
ENCRYPTION_KEY=your-super-secret-encryption-key-32-chars

# AI Provider API Keys (replace with your actual keys)
OPENAI_API_KEY=your-openai-api-key-here
ANTHROPIC_API_KEY=your-anthropic-api-key-here
GOOGLE_API_KEY=your-google-api-key-here
HUGGINGFACE_API_KEY=your-huggingface-api-key-here
COHERE_API_KEY=your-cohere-api-key-here
TOGETHER_API_KEY=your-together-api-key-here

# Telegram Configuration
TELEGRAM_BOT_TOKEN=your_telegram_bot_token_here
TELEGRAM_API_ID=your_telegram_api_id_here
TELEGRAM_API_HASH=your_telegram_api_hash_here
TELEGRAM_PHONE_NUMBER=+1234567890

# Monitoring Configuration
PROMETHEUS_ENDPOINT=http://localhost:9090
JAEGER_ENDPOINT=http://localhost:14268/api/traces

# System Configuration
LOG_LEVEL=INFO
ENVIRONMENT=development
"""
    
    with open(env_file, 'w') as f:
        f.write(env_content)
    
    print(f"Environment file created: {env_file}")
    print("Please edit .env file and add your actual API keys")

async def main():
    """Main setup function"""
    print("=" * 60)
    print("PostgreSQL Database Setup for AI Intelligence System")
    print("=" * 60)
    
    print(f"Database Host: {DB_HOST}")
    print(f"Database Port: {DB_PORT}")
    print(f"Database Name: {DB_NAME}")
    print(f"Database User: {DB_USER}")
    print()
    
    # Check if admin credentials are provided
    if ADMIN_PASSWORD == 'postgres':
        print("WARNING: Using default PostgreSQL admin password!")
        print("Please set POSTGRES_ADMIN_PASSWORD environment variable for production use.")
        print()
    
    try:
        # Step 1: Create database and user
        print("Step 1: Creating database and user...")
        await create_database_and_user()
        print()
        
        # Step 2: Test connection
        print("Step 2: Testing database connection...")
        await test_connection()
        print()
        
        # Step 3: Create initial schema
        print("Step 3: Creating initial schema...")
        await create_initial_schema()
        print()
        
        # Step 4: Create environment file
        print("Step 4: Creating environment configuration...")
        await setup_environment_file()
        print()
        
        print("=" * 60)
        print("PostgreSQL setup completed successfully!")
        print("=" * 60)
        print()
        print("Next steps:")
        print("1. Edit .env file with your actual API keys")
        print("2. Run: python setup.py")
        print("3. Run: python run_system.py")
        print()
        
    except Exception as e:
        print(f"Setup failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())
