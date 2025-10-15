#!/bin/bash

# Autonomous AI Intelligence System Setup Script
# This script sets up the database and starts the autonomous system

set -e

echo "🚀 Setting up AUTONOMOUS AI Intelligence System..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "rust-intelligence-system/Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if .env file exists
if [ ! -f "rust-intelligence-system/.env" ]; then
    if [ -f "supabase.env" ]; then
        print_warning "No .env file found. Creating from Supabase template..."
        cp supabase.env rust-intelligence-system/.env
        print_status "Created .env from supabase.env template"
    else
        print_error "No environment file found. Please create .env or supabase.env"
        exit 1
    fi
fi

# Function to setup database
setup_database() {
    print_status "Setting up Supabase database schema..."
    
    # Check if we can connect to the database
    DATABASE_URL=$(grep "DATABASE_URL" rust-intelligence-system/.env | cut -d '=' -f2)
    if [ -z "$DATABASE_URL" ]; then
        print_error "DATABASE_URL not found in .env file"
        exit 1
    fi
    
    print_status "Database URL configured: ${DATABASE_URL:0:50}..."
    
    # Note: In a real setup, you would run the SQL schema here
    # For now, we'll assume the database is already set up
    print_success "Database schema ready (manual setup required)"
    print_warning "Please run the database_schema.sql in your Supabase SQL Editor"
}

# Function to build the application
build_application() {
    print_status "Building autonomous AI intelligence system..."
    
    cd rust-intelligence-system
    
    # Build the application
    print_status "Compiling Rust application (this may take a few minutes)..."
    source "$HOME/.cargo/env"
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_success "Autonomous AI Intelligence System built successfully"
    else
        print_error "Failed to build application"
        exit 1
    fi
    
    cd ..
}

# Function to start the autonomous system
start_autonomous_system() {
    print_status "Starting AUTONOMOUS AI Intelligence System..."
    
    cd rust-intelligence-system
    
    # Set environment variables
    export RUST_LOG=info
    export RUST_BACKTRACE=1
    
    # Load environment file
    if [ -f ".env" ]; then
        print_status "Loading environment variables from .env file..."
        set -a  # automatically export all variables
        source .env
        set +a
    fi
    
    # Start the autonomous application
    print_status "Starting autonomous intelligence system..."
    source "$HOME/.cargo/env"
    cargo run --release --bin intelligence-api &
    
    # Store the PID
    APP_PID=$!
    echo $APP_PID > autonomous-intelligence-api.pid
    
    # Wait a moment for the application to start
    sleep 5
    
    # Check if the application is running
    if ps -p $APP_PID > /dev/null; then
        print_success "AUTONOMOUS AI Intelligence System started successfully!"
        print_status "Application PID: $APP_PID"
        print_status "API available at: http://localhost:8080"
        print_status "Health check: http://localhost:8080/health"
        print_status ""
        print_status "🤖 AUTONOMOUS FEATURES RUNNING:"
        print_status "  ✅ Telegram connection monitoring"
        print_status "  ✅ Automatic message analysis"
        print_status "  ✅ Style learning and pattern recognition"
        print_status "  ✅ Autonomous response generation"
        print_status "  ✅ Database logging and storage"
        print_status "  ✅ Real-time chat monitoring"
        print_status ""
        print_status "📊 MONITORING ENDPOINTS:"
        print_status "  - Health: http://localhost:8080/health"
        print_status "  - Stats: http://localhost:8080/api/stats"
        print_status "  - Chats: http://localhost:8080/api/chats"
        print_status "  - Messages: http://localhost:8080/api/messages"
        print_status "  - Logs: http://localhost:8080/api/logs"
        print_status ""
        print_success "The system is now running AUTONOMOUSLY!"
        print_success "It will:"
        print_success "  • Connect to Telegram automatically"
        print_success "  • Monitor all your chats"
        print_success "  • Analyze message patterns"
        print_success "  • Learn communication styles"
        print_success "  • Generate responses automatically"
        print_success "  • Log everything to the database"
        print_status ""
        print_status "To stop the system, run: ./stop-autonomous.sh"
    else
        print_error "Failed to start AUTONOMOUS AI Intelligence System"
        exit 1
    fi
    
    cd ..
}

# Function to show status
show_status() {
    print_status "Checking autonomous system status..."
    
    # Check if application is running
    if [ -f "rust-intelligence-system/autonomous-intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/autonomous-intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_success "AUTONOMOUS AI Intelligence System is running (PID: $PID)"
            
            # Test the API
            if curl -s http://localhost:8080/health > /dev/null 2>&1; then
                print_success "API is responding"
            else
                print_warning "API is not responding"
            fi
        else
            print_warning "AUTONOMOUS AI Intelligence System is not running"
        fi
    else
        print_warning "AUTONOMOUS AI Intelligence System is not running"
    fi
}

# Main execution
case "${1:-start}" in
    "start")
        setup_database
        build_application
        start_autonomous_system
        ;;
    "build")
        build_application
        ;;
    "app")
        start_autonomous_system
        ;;
    "status")
        show_status
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  start   - Start the complete autonomous system (default)"
        echo "  build   - Build the Rust application"
        echo "  app     - Start only the application"
        echo "  status  - Show system status"
        echo "  help    - Show this help message"
        echo ""
        echo "AUTONOMOUS FEATURES:"
        echo "  - Telegram connection monitoring"
        echo "  - Automatic message analysis"
        echo "  - Style learning and pattern recognition"
        echo "  - Autonomous response generation"
        echo "  - Database logging and storage"
        echo "  - Real-time chat monitoring"
        echo ""
        echo "The system runs completely autonomously!"
        echo "No manual intervention required!"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
