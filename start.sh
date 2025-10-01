#!/bin/bash

# AI Intelligence System Startup Script
# This script starts the Rust intelligence system with all dependencies

set -e  # Exit on any error

echo "🚀 Starting AI Intelligence System..."

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

# Check if Docker is running (skip for help command)
if [ "$1" != "help" ] && [ "$1" != "status" ]; then
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
fi

# Check if Rust is installed (skip for help command)
if [ "$1" != "help" ] && [ "$1" != "status" ]; then
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is not installed. Please install Rust first:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
fi

# Check if .env file exists
if [ ! -f "production.env" ]; then
    print_warning "No production.env file found. Creating from template..."
    if [ -f "rust-intelligence-system/config.template.yaml" ]; then
        print_status "Please copy production.env to .env and configure your settings"
        print_status "Example: cp production.env .env"
        print_status "Then edit .env with your actual API keys and configuration"
    fi
fi

# Function to start dependencies
start_dependencies() {
    print_status "Starting infrastructure dependencies..."
    
    # Start PostgreSQL, Redis, Kafka, and monitoring stack
    cd rust-intelligence-system
    
    print_status "Starting PostgreSQL, Redis, Kafka, and monitoring services..."
    docker-compose up -d postgres redis kafka zookeeper prometheus grafana jaeger elasticsearch kibana
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 10
    
    # Check if services are running
    if docker-compose ps | grep -q "Up"; then
        print_success "Infrastructure services started successfully"
    else
        print_error "Failed to start infrastructure services"
        exit 1
    fi
    
    cd ..
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
    print_status "Starting AI Intelligence System..."
    
    cd rust-intelligence-system
    
    # Set environment variables
    export RUST_LOG=info
    export RUST_BACKTRACE=1
    
    # Load environment file if it exists
    if [ -f "../.env" ]; then
        print_status "Loading environment variables from .env file..."
        set -a  # automatically export all variables
        source ../.env
        set +a
    elif [ -f "../production.env" ]; then
        print_warning "Using production.env template. Please copy to .env and configure."
        set -a
        source ../production.env
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
        print_success "AI Intelligence System started successfully!"
        print_status "Application PID: $APP_PID"
        print_status "API available at: http://localhost:8080"
        print_status "Health check: http://localhost:8080/health"
        print_status "Metrics: http://localhost:8080/metrics"
        print_status ""
        print_status "Monitoring dashboards:"
        print_status "  - Grafana: http://localhost:3000 (admin/admin)"
        print_status "  - Prometheus: http://localhost:9090"
        print_status "  - Jaeger: http://localhost:16686"
        print_status "  - Kibana: http://localhost:5601"
        print_status ""
        print_status "To stop the system, run: ./stop.sh"
    else
        print_error "Failed to start AI Intelligence System"
        exit 1
    fi
    
    cd ..
}

# Function to show status
show_status() {
    print_status "Checking system status..."
    
    # Check if application is running
    if [ -f "rust-intelligence-system/intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_success "AI Intelligence System is running (PID: $PID)"
        else
            print_warning "AI Intelligence System is not running"
        fi
    else
        print_warning "AI Intelligence System is not running"
    fi
    
    # Check Docker services
    cd rust-intelligence-system
    print_status "Docker services status:"
    docker-compose ps
    cd ..
}

# Main execution
case "${1:-start}" in
    "start")
        start_dependencies
        build_application
        start_application
        ;;
    "deps")
        start_dependencies
        ;;
    "build")
        build_application
        ;;
    "app")
        start_application
        ;;
    "status")
        show_status
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  start   - Start the complete system (default)"
        echo "  deps    - Start only dependencies"
        echo "  build   - Build the Rust application"
        echo "  app     - Start only the application"
        echo "  status  - Show system status"
        echo "  help    - Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
