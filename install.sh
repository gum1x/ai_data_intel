#!/bin/bash

echo "Setting up AI Intelligence System..."

python3 -m venv venv
source venv/bin/activate

pip install --upgrade pip
pip install -r requirements.txt

mkdir -p data logs models cache temp

echo "Installing PostgreSQL dependencies..."
pip install asyncpg psycopg2-binary alembic

echo "Note: PostgreSQL database setup will be handled by the application"
echo "Please ensure PostgreSQL is installed and running before starting the system"

echo "Setup completed successfully!"
echo "To run the system:"
echo "1. source venv/bin/activate"
echo "2. python3 run_system.py"
