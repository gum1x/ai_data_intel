#!/bin/bash

# FREE AI Intelligence System Startup Script
# This script starts the system with minimal resources and zero external costs

set -e  # Exit on any error

echo "🆓 Starting FREE AI Intelligence System..."

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
if [ ! -f ".env" ]; then
    if [ -f "free.env" ]; then
        print_warning "No .env file found. Creating from free template..."
        cp free.env .env
        print_status "Created .env from free.env template"
        print_warning "This is a FREE configuration with limited features"
        print_status "Edit .env if you want to add paid API keys later"
    else
        print_error "No environment file found. Please create .env or free.env"
        exit 1
    fi
fi

# Function to install Ollama (free local AI)
install_ollama() {
    print_status "Installing Ollama for free local AI models..."
    
    if command -v ollama &> /dev/null; then
        print_success "Ollama is already installed"
    else
        print_status "Installing Ollama..."
        curl -fsSL https://ollama.ai/install.sh | sh
        
        if [ $? -eq 0 ]; then
            print_success "Ollama installed successfully"
        else
            print_error "Failed to install Ollama"
            exit 1
        fi
    fi
    
    # Start Ollama service
    print_status "Starting Ollama service..."
    ollama serve &
    sleep 5
    
    # Pull a small model
    print_status "Downloading Llama2 7B model (this may take a while)..."
    ollama pull llama2:7b
    
    print_success "Ollama setup complete"
}

# Function to start free dependencies
start_dependencies() {
    print_status "Starting FREE infrastructure dependencies..."
    
    cd rust-intelligence-system
    
    print_status "Starting PostgreSQL, Redis, and Ollama..."
    docker-compose -f docker-compose.free.yml up -d postgres redis ollama
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 15
    
    # Check if services are running
    if docker-compose -f docker-compose.free.yml ps | grep -q "Up"; then
        print_success "Infrastructure services started successfully"
    else
        print_error "Failed to start infrastructure services"
        exit 1
    fi
    
    cd ..
}

# Function to start monitoring (optional)
start_monitoring() {
    print_status "Starting basic monitoring (optional)..."
    
    cd rust-intelligence-system
    
    print_status "Starting Prometheus and Grafana..."
    docker-compose -f docker-compose.free.yml up -d prometheus grafana
    
    print_success "Monitoring services started"
    
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
    print_status "Starting FREE AI Intelligence System..."
    
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
        print_success "FREE AI Intelligence System started successfully!"
        print_status "Application PID: $APP_PID"
        print_status "API available at: http://localhost:8080"
        print_status "Health check: http://localhost:8080/health"
        print_status ""
        print_status "FREE Services:"
        print_status "  - API: http://localhost:8080"
        print_status "  - Ollama AI: http://localhost:11434"
        print_status "  - Grafana: http://localhost:3000 (admin/admin)"
        print_status "  - Prometheus: http://localhost:9090"
        print_status ""
        print_warning "This is a FREE version with limited features:"
        print_warning "  - No paid AI APIs (using local Ollama)"
        print_warning "  - Reduced processing limits"
        print_warning "  - Basic monitoring only"
        print_status ""
        print_status "To stop the system, run: ./stop-free.sh"
    else
        print_error "Failed to start FREE AI Intelligence System"
        exit 1
    fi
    
    cd ..
}

# Function to show status
show_status() {
    print_status "Checking FREE system status..."
    
    # Check if application is running
    if [ -f "rust-intelligence-system/intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_success "FREE AI Intelligence System is running (PID: $PID)"
        else
            print_warning "FREE AI Intelligence System is not running"
        fi
    else
        print_warning "FREE AI Intelligence System is not running"
    fi
    
    # Check Docker services
    cd rust-intelligence-system
    print_status "Docker services status:"
    docker-compose -f docker-compose.free.yml ps
    cd ..
    
    # Check Ollama
    if command -v ollama &> /dev/null; then
        print_status "Ollama models:"
        ollama list
    fi
}

# Main execution
case "${1:-start}" in
    "start")
        install_ollama
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
    "monitoring")
        start_monitoring
        ;;
    "ollama")
        install_ollama
        ;;
    "status")
        show_status
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  start      - Start the complete FREE system (default)"
        echo "  deps       - Start only dependencies"
        echo "  build      - Build the Rust application"
        echo "  app        - Start only the application"
        echo "  monitoring - Start monitoring services"
        echo "  ollama     - Install and setup Ollama"
        echo "  status     - Show system status"
        echo "  help       - Show this help message"
        echo ""
        echo "FREE Features:"
        echo "  - Local AI models via Ollama"
        echo "  - No external API costs"
        echo "  - Minimal resource usage"
        echo "  - Basic monitoring"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
