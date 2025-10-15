#!/bin/bash

# NO-DOCKER AI Intelligence System Startup Script
# This script runs the system without Docker dependencies

set -e  # Exit on any error

echo "🚀 Starting NO-DOCKER AI Intelligence System..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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
if [ ! -f ".env" ]; then
    if [ -f "supabase.env" ]; then
        print_warning "No .env file found. Creating from Supabase template..."
        cp supabase.env .env
        print_status "Created .env from supabase.env template"
    else
        print_error "No environment file found. Please create .env or supabase.env"
        exit 1
    fi
fi

# Function to check external services
check_external_services() {
    print_status "Checking external services..."
    
    # Check Redis Cloud
    if redis-cli -u redis://default:nMeJnBpTATLVt2asHpl7s5ebtv3oC156@redis-12632.c257.us-east-1-3.ec2.redns.redis-cloud.com:12632 ping > /dev/null 2>&1; then
        print_success "Redis Cloud: Connected"
    else
        print_error "Redis Cloud: Connection failed"
        exit 1
    fi
    
    # Check Supabase
    if curl -s -H "apikey: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImVpYnRybHBvbmVreXJ3YnFjb3R0Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjA1MDE1MTgsImV4cCI6MjA3NjA3NzUxOH0.X2o7dF6YRfoDtWI7YkVg77v3I4f0bwFBqq8y8OGTpsY" https://eibtrlponekyrwbqcott.supabase.co/rest/v1/ > /dev/null 2>&1; then
        print_success "Supabase: Connected"
    else
        print_error "Supabase: Connection failed"
        exit 1
    fi
    
    # Check Ollama
    if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        print_success "Ollama AI: Running"
    else
        print_warning "Ollama AI: Not running (optional)"
    fi
}

# Function to build the Rust application
build_application() {
    print_status "Building Rust intelligence system..."
    
    cd rust-intelligence-system
    
    # Build the application
    print_status "Compiling Rust application (this may take a few minutes)..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_success "Rust application built successfully"
    else
        print_error "Failed to build Rust application"
        exit 1
    fi
    
    cd ..
}

# Function to start the application
start_application() {
    print_status "Starting NO-DOCKER AI Intelligence System..."
    
    cd rust-intelligence-system
    
    # Set environment variables
    export RUST_LOG=info
    export RUST_BACKTRACE=1
    
    # Load environment file
    if [ -f "../.env" ]; then
        print_status "Loading environment variables from .env file..."
        set -a  # automatically export all variables
        source ../.env
        set +a
    fi
    
    # Start the application
    print_status "Starting intelligence-api..."
    cargo run --bin intelligence-api --release &
    
    # Store the PID
    APP_PID=$!
    echo $APP_PID > intelligence-api.pid
    
    # Wait a moment for the application to start
    sleep 5
    
    # Check if the application is running
    if ps -p $APP_PID > /dev/null; then
        print_success "NO-DOCKER AI Intelligence System started successfully!"
        print_status "Application PID: $APP_PID"
        print_status "API available at: http://localhost:8080"
        print_status "Health check: http://localhost:8080/health"
        print_status ""
        print_status "Services Running:"
        print_status "  - API: http://localhost:8080"
        print_status "  - Supabase: https://eibtrlponekyrwbqcott.supabase.co"
        print_status "  - Redis Cloud: Connected"
        print_status "  - Ollama AI: http://localhost:11434"
        print_status ""
        print_success "NO-DOCKER Features:"
        print_success "  - Database: Supabase (cloud)"
        print_success "  - Cache: Redis Cloud (external)"
        print_success "  - AI: Ollama (local)"
        print_success "  - Auth: Supabase Auth"
        print_success "  - Storage: Supabase Storage"
        print_status ""
        print_status "To stop the system, run: ./stop-no-docker.sh"
    else
        print_error "Failed to start NO-DOCKER AI Intelligence System"
        exit 1
    fi
    
    cd ..
}

# Function to show status
show_status() {
    print_status "Checking NO-DOCKER system status..."
    
    # Check if application is running
    if [ -f "rust-intelligence-system/intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_success "NO-DOCKER AI Intelligence System is running (PID: $PID)"
        else
            print_warning "NO-DOCKER AI Intelligence System is not running"
        fi
    else
        print_warning "NO-DOCKER AI Intelligence System is not running"
    fi
    
    # Check external services
    check_external_services
}

# Main execution
case "${1:-start}" in
    "start")
        check_external_services
        build_application
        start_application
        ;;
    "build")
        build_application
        ;;
    "app")
        start_application
        ;;
    "check")
        check_external_services
        ;;
    "status")
        show_status
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  start   - Start the complete NO-DOCKER system (default)"
        echo "  build   - Build the Rust application"
        echo "  app     - Start only the application"
        echo "  check   - Check external services"
        echo "  status  - Show system status"
        echo "  help    - Show this help message"
        echo ""
        echo "NO-DOCKER Features:"
        echo "  - Supabase (cloud database)"
        echo "  - Redis Cloud (external cache)"
        echo "  - Ollama (local AI)"
        echo "  - No Docker required!"
        echo "  - Still 100% free!"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
