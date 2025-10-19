#!/bin/bash

echo "🧪 Testing graceful shutdown functionality..."
echo ""

# Check if system is running
echo "1. Checking system status..."
curl -s http://localhost:8080/api/stats | jq '.data.is_running'

echo ""
echo "2. Sending Ctrl+C signal to test graceful shutdown..."
echo "   (This will stop the system gracefully)"

# Find the process and send SIGINT
PID=$(pgrep -f intelligence-api)
if [ ! -z "$PID" ]; then
    echo "   Found process PID: $PID"
    kill -INT $PID
    echo "   Signal sent! System should shut down gracefully..."
    
    # Wait a moment and check if it's still running
    sleep 3
    if pgrep -f intelligence-api > /dev/null; then
        echo "   ❌ System still running - graceful shutdown may have failed"
    else
        echo "   ✅ System stopped gracefully!"
    fi
else
    echo "   ❌ No intelligence-api process found"
fi

echo ""
echo "3. Testing if system responds after shutdown..."
sleep 2
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "   ❌ System still responding - shutdown failed"
else
    echo "   ✅ System no longer responding - shutdown successful"
fi
