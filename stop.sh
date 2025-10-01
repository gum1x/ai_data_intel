#!/bin/bash

# AI Intelligence System Stop Script
# This script stops the Rust intelligence system and all dependencies

set -e  # Exit on any error

echo "🛑 Stopping AI Intelligence System..."

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

# Function to stop the application
stop_application() {
    print_status "Stopping AI Intelligence System application..."
    
    if [ -f "rust-intelligence-system/intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_status "Stopping application (PID: $PID)..."
            kill $PID
            sleep 2
            
            # Force kill if still running
            if ps -p $PID > /dev/null; then
                print_warning "Application didn't stop gracefully, force killing..."
                kill -9 $PID
            fi
            
            print_success "Application stopped"
        else
            print_warning "Application was not running"
        fi
        
        # Remove PID file
        rm -f rust-intelligence-system/intelligence-api.pid
    else
        print_warning "No PID file found, application may not be running"
    fi
}

# Function to stop dependencies
stop_dependencies() {
    print_status "Stopping infrastructure dependencies..."
    
    cd rust-intelligence-system
    
    print_status "Stopping Docker services..."
    docker-compose down
    
    print_success "Infrastructure services stopped"
    
    cd ..
}

# Function to clean up
cleanup() {
    print_status "Cleaning up..."
    
    # Remove any remaining PID files
    rm -f rust-intelligence-system/intelligence-api.pid
    
    print_success "Cleanup completed"
}

# Function to show status
show_status() {
    print_status "Checking system status..."
    
    # Check if application is running
    if [ -f "rust-intelligence-system/intelligence-api.pid" ]; then
        PID=$(cat rust-intelligence-system/intelligence-api.pid)
        if ps -p $PID > /dev/null; then
            print_warning "AI Intelligence System is still running (PID: $PID)"
        else
            print_success "AI Intelligence System is stopped"
        fi
    else
        print_success "AI Intelligence System is stopped"
    fi
    
    # Check Docker services
    cd rust-intelligence-system
    print_status "Docker services status:"
    docker-compose ps
    cd ..
}

# Main execution
case "${1:-stop}" in
    "stop")
        stop_application
        stop_dependencies
        cleanup
        ;;
    "app")
        stop_application
        ;;
    "deps")
        stop_dependencies
        ;;
    "cleanup")
        cleanup
        ;;
    "status")
        show_status
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  stop    - Stop the complete system (default)"
        echo "  app     - Stop only the application"
        echo "  deps    - Stop only dependencies"
        echo "  cleanup - Clean up PID files and temporary data"
        echo "  status  - Show system status"
        echo "  help    - Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
