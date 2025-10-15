
# 🧠 AI Intelligence System

A production-ready, enterprise-grade intelligence platform built with Rust, featuring advanced analytics, machine learning, real-time processing, and robust security.

## 🚀 Quick Start

### 🆓 FREE Version (Zero Cost)
```bash
# Copy FREE environment template
cp free.env .env

# Start FREE system (no API keys needed!)
chmod +x start-free.sh stop-free.sh
./start-free.sh
```

### 💰 Production Version (With Paid APIs)
```bash
# Copy production environment template
cp production.env .env

# Edit with your actual API keys
nano .env

# Start production system
chmod +x start.sh stop.sh
./start.sh
```

### Prerequisites
- Rust 1.75+
- Docker & Docker Compose
- 4+ CPU cores, 4GB+ RAM (FREE) / 8GB+ RAM (Production)

### Access Services
- **API**: http://localhost:8080
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Ollama AI** (FREE): http://localhost:11434

## 📚 Documentation

- **[FREE Setup Guide](FREE_SETUP_GUIDE.md)** - Zero cost setup and configuration
- **[Production Setup Guide](STARTUP_GUIDE.md)** - Complete setup with paid APIs
- **[Rust System README](rust-intelligence-system/README.md)** - Technical documentation

## 🏗️ Architecture

This system consists of:

- **Rust Intelligence System** - Core analytics and processing engine
- **Docker Infrastructure** - PostgreSQL, Redis, Kafka, monitoring stack
- **Production Configuration** - Environment variables and security settings

## 🔧 Configuration

### Required Environment Variables
```bash
# Security (REQUIRED)
JWT_SECRET=your_32_char_secret_here
ENCRYPTION_KEY=your_32_char_key_here

# Database (REQUIRED)
POSTGRES_PASSWORD=your_secure_password
REDIS_PASSWORD=your_redis_password

# AI Providers (REQUIRED - at least one)
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
GOOGLE_API_KEY=your_google_key
```

## 🛠️ Commands

### FREE Version
```bash
# Start FREE system
./start-free.sh

# Stop FREE system
./stop-free.sh

# Check status
./start-free.sh status

# Install Ollama AI
./start-free.sh ollama
```

### Production Version
```bash
# Start production system
./start.sh

# Stop production system
./stop.sh

# Check status
./start.sh status

# Start only dependencies
./start.sh deps
```

## 🔒 Security

- JWT-based authentication
- End-to-end encryption
- Rate limiting
- Audit logging
- RBAC system

## 📊 Monitoring

- Prometheus metrics
- Grafana dashboards
- Jaeger tracing
- Health checks
- Performance monitoring

## ⚠️ Disclaimer
This system is provided for educational purposes only. It is not intended for practical use. Its sole purpose is to allow people to review the codebase and understand how systems like this can be built and used. It is not optimized for efficiency, and should not be considered the best or recommended approach for real-world applications.
