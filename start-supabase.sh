#!/bin/bash

# SUPABASE AI Intelligence System Startup Script
# This script starts the system with Supabase integration

set -e  # Exit on any error

echo "🚀 Starting SUPABASE AI Intelligence System..."

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
if [ "$1" != "help" ] && [ "$1" != "status" ] && [ "$1" != "validate" ]; then
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
fi

# Check if Rust is installed (skip for help command)
if [ "$1" != "help" ] && [ "$1" != "status" ] && [ "$1" != "validate" ]; then
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is not installed. Please install Rust first:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
fi

# Check if .env file exists
if [ ! -f ".env" ]; then
    if [ -f "supabase.env" ]; then
        print_warning "No .env file found. Creating from Supabase template..."
        cp supabase.env .env
        print_status "Created .env from supabase.env template"
        print_warning "Please configure your Supabase project settings in .env"
        print_status "You need to:"
        print_status "1. Create a Supabase project at https://supabase.com"
        print_status "2. Get your project URL and API keys"
        print_status "3. Update SUPABASE_URL, SUPABASE_ANON_KEY, etc. in .env"
        print_status "4. Run this script again"
        exit 1
    else
        print_error "No environment file found. Please create .env or supabase.env"
        exit 1
    fi
fi

# Function to validate Supabase configuration
validate_supabase_config() {
    print_status "Validating Supabase configuration..."
    
    # Check if required Supabase variables are set
    if ! grep -q "SUPABASE_URL=https://.*\.supabase\.co" .env; then
        print_error "SUPABASE_URL not properly configured in .env"
        print_status "Please set SUPABASE_URL=https://your-project-id.supabase.co"
        exit 1
    fi
    
    if ! grep -q "SUPABASE_ANON_KEY=.*" .env || grep -q "SUPABASE_ANON_KEY=your_supabase_anon_key_here" .env; then
        print_error "SUPABASE_ANON_KEY not properly configured in .env"
        print_status "Please set your actual Supabase anon key"
        exit 1
    fi
    
    if ! grep -q "SUPABASE_SERVICE_ROLE_KEY=.*" .env || grep -q "SUPABASE_SERVICE_ROLE_KEY=your_supabase_service_role_key_here" .env; then
        print_error "SUPABASE_SERVICE_ROLE_KEY not properly configured in .env"
        print_status "Please set your actual Supabase service role key"
        exit 1
    fi
    
    print_success "Supabase configuration validated"
}

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

# Function to start Supabase dependencies
start_dependencies() {
    print_status "Starting Supabase infrastructure dependencies..."
    
    cd rust-intelligence-system
    
    print_status "Starting Redis and Ollama..."
    docker-compose -f docker-compose.supabase.yml up -d redis ollama
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 15
    
    # Check if services are running
    if docker-compose -f docker-compose.supabase.yml ps | grep -q "Up"; then
        print_success "Infrastructure services started successfully"
    else
        print_error "Failed to start infrastructure services"
        exit 1
    fi
    
    cd ..
}

# Function to start monitoring (optional)
start_monitoring() {
    print_status "Starting Supabase monitoring..."
    
    cd rust-intelligence-system
    
    print_status "Starting Prometheus and Grafana..."
    docker-compose -f docker-compose.supabase.yml up -d prometheus grafana
    
    print_success "Monitoring services started"
    
    cd ..
}

# Function to build the Rust application
build_application() {
    print_status "Building Rust intelligence system with Supabase integration..."
    
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
    print_status "Starting SUPABASE AI Intelligence System..."
    
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
        print_success "SUPABASE AI Intelligence System started successfully!"
        print_status "Application PID: $APP_PID"
        print_status "API available at: http://localhost:8080"
        print_status "Health check: http://localhost:8080/health"
        print_status ""
        print_status "SUPABASE Services:"
        print_status "  - API: http://localhost:8080"
        print_status "  - Supabase Dashboard: https://supabase.com/dashboard"
        print_status "  - Ollama AI: http://localhost:11434"
        print_status "  - Grafana: http://localhost:3000 (admin/admin)"
        print_status "  - Prometheus: http://localhost:9090"
        print_status ""
        print_success "SUPABASE Features Enabled:"
        print_success "  - Database: PostgreSQL (Supabase)"
        print_success "  - Authentication: Supabase Auth"
        print_success "  - Storage: Supabase Storage (1GB free)"
        print_success "  - Realtime: Supabase Realtime"
        print_success "  - Local AI: Ollama models"
        print_status ""
        print_status "To stop the system, run: ./stop-supabase.sh"
    else
        print_error "Failed to start SUPABASE AI Intelligence System"
        exit 1
    fi
    
    cd ..
}

# Function to show status
show_status() {
    print_status "Checking SUPABASE system status..."
    
    # Check if application is running
    if [ -f "rust-intelligence-system/intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_success "SUPABASE AI Intelligence System is running (PID: $PID)"
        else
            print_warning "SUPABASE AI Intelligence System is not running"
        fi
    else
        print_warning "SUPABASE AI Intelligence System is not running"
    fi
    
    # Check Docker services
    cd rust-intelligence-system
    print_status "Docker services status:"
    docker-compose -f docker-compose.supabase.yml ps
    cd ..
    
    # Check Ollama
    if command -v ollama &> /dev/null; then
        print_status "Ollama models:"
        ollama list
    fi
    
    # Check Supabase connection
    if [ -f ".env" ]; then
        SUPABASE_URL=$(grep "SUPABASE_URL=" .env | cut -d'=' -f2)
        print_status "Supabase URL: $SUPABASE_URL"
    fi
}

# Main execution
case "${1:-start}" in
    "start")
        validate_supabase_config
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
    "validate")
        validate_supabase_config
        ;;
    "status")
        show_status
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  start      - Start the complete SUPABASE system (default)"
        echo "  deps       - Start only dependencies"
        echo "  build      - Build the Rust application"
        echo "  app        - Start only the application"
        echo "  monitoring - Start monitoring services"
        echo "  ollama     - Install and setup Ollama"
        echo "  validate   - Validate Supabase configuration"
        echo "  status     - Show system status"
        echo "  help       - Show this help message"
        echo ""
        echo "SUPABASE Features:"
        echo "  - PostgreSQL Database (Supabase)"
        echo "  - Authentication (Supabase Auth)"
        echo "  - File Storage (Supabase Storage)"
        echo "  - Real-time subscriptions (Supabase Realtime)"
        echo "  - Local AI models (Ollama)"
        echo "  - Free tier: 500MB DB, 1GB storage, 50k users"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
