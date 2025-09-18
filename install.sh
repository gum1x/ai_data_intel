#!/bin/bash

echo "Setting up AI Intelligence System..."

python3 -m venv venv
source venv/bin/activate

pip install --upgrade pip
pip install -r requirements.txt

mkdir -p data logs models cache temp

python3 -c "
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
print('Database setup completed')
"

echo "Setup completed successfully!"
echo "To run the system:"
echo "1. source venv/bin/activate"
echo "2. python3 run_system.py"
