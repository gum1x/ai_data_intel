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

def setup_database():
    try:
        import sqlite3
        conn = sqlite3.connect('ai_intelligence.db')
        cursor = conn.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS system_status (
                id INTEGER PRIMARY KEY,
                component TEXT,
                status TEXT,
                last_update TEXT
            )
        ''')
        
        conn.commit()
        conn.close()
        print("Database setup completed")
    except Exception as e:
        print(f"Database setup failed: {e}")

def main():
    print("Setting up AI Intelligence System...")
    
    if not install_requirements():
        print("Setup failed at requirements installation")
        return
    
    create_directories()
    setup_database()
    
    print("Setup completed successfully!")
    print("Run 'python run_system.py' to start the system")

if __name__ == "__main__":
    main()
