
# 🧠 AI Intelligence System

A production-ready, enterprise-grade intelligence platform built with Rust, featuring advanced analytics, machine learning, real-time processing, and robust security.

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+
- Docker & Docker Compose
- 4+ CPU cores, 8GB+ RAM

### 1. Environment Setup
```bash
# Copy environment template
cp production.env .env

# Edit with your actual values
nano .env
```

### 2. Start the System
```bash
# Make scripts executable
chmod +x start.sh stop.sh

# Start complete system
./start.sh
```

### 3. Access Services
- **API**: http://localhost:8080
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686

## 📚 Documentation

- **[Startup Guide](STARTUP_GUIDE.md)** - Complete setup and configuration
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

```bash
# Start system
./start.sh

# Stop system
./stop.sh

# Check status
./start.sh status

# Start only dependencies
./start.sh deps

# Build only
./start.sh build
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
