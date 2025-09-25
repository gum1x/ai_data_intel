#!/usr/bin/env python3

import subprocess
import sys
import os

def install_requirements():
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "-r", "requirements.txt"])
        print("Requirements installed successfully")
    except subprocess.CalledProcessError as e:
        print(f"Failed to install requirements: {e}")
        return False
    return True

def create_directories():
    directories = [
        "data",
        "logs",
        "models",
        "cache",
        "temp"
    ]
    
    for directory in directories:
        os.makedirs(directory, exist_ok=True)
        print(f"Created directory: {directory}")

async def setup_database():
    try:
        from database_manager import DatabaseManager
        
        # Database configuration
        db_config = {
            'host': os.getenv('DATABASE_HOST', 'localhost'),
            'port': int(os.getenv('DATABASE_PORT', 5432)),
            'name': os.getenv('DATABASE_NAME', 'intelligence'),
            'username': os.getenv('DATABASE_USER', 'intelligence'),
            'password': os.getenv('DATABASE_PASSWORD', 'password'),
            'max_connections': 100,
            'connection_timeout': 30
        }
        
        # Initialize database manager
        db_manager = DatabaseManager({'database': db_config})
        await db_manager.initialize()
        
        print("PostgreSQL database setup completed")
        
    except Exception as e:
        print(f"Database setup failed: {e}")
        print("Please ensure PostgreSQL is running and credentials are correct")

async def main():
    print("Setting up AI Intelligence System...")
    
    if not install_requirements():
        print("Setup failed at requirements installation")
        return
    
    create_directories()
    await setup_database()
    
    print("Setup completed successfully!")
    print("Run 'python run_system.py' to start the system")

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
