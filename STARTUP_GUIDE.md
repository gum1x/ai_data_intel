# 🚀 AI Intelligence System - Startup Guide

This guide will help you get the AI Intelligence System up and running in production.

## 📋 Prerequisites

Before starting, ensure you have the following installed:

### Required Software
- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Docker & Docker Compose** - [Install Docker](https://docs.docker.com/get-docker/)
- **Git** - [Install Git](https://git-scm.com/downloads)

### System Requirements
- **Minimum**: 4 CPU cores, 8GB RAM, 50GB storage
- **Recommended**: 8 CPU cores, 16GB RAM, 100GB storage
- **Production**: 16+ CPU cores, 32GB+ RAM, 500GB+ storage

## 🔧 Quick Start

### 1. Environment Setup

```bash
# Copy the production environment template
cp production.env .env

# Edit the .env file with your actual values
nano .env
```

**⚠️ IMPORTANT**: Update these critical values in your `.env` file:

```bash
# Security (REQUIRED - Generate secure random strings)
JWT_SECRET=your_super_secret_jwt_key_must_be_32_chars_minimum_length
ENCRYPTION_KEY=your_super_secret_encryption_key_32_chars

# Database (REQUIRED)
POSTGRES_PASSWORD=your_secure_database_password
REDIS_PASSWORD=your_redis_password

# AI Provider API Keys (REQUIRED - At least one)
OPENAI_API_KEY=your_openai_api_key_here
ANTHROPIC_API_KEY=your_anthropic_api_key_here
GOOGLE_API_KEY=your_google_api_key_here

# Telegram (OPTIONAL - For Telegram data collection)
TELEGRAM_API_ID=your_telegram_api_id
TELEGRAM_API_HASH=your_telegram_api_hash
TELEGRAM_PHONE_NUMBER=+1234567890

# Blockchain (OPTIONAL - For blockchain analysis)
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your_infura_project_id
POLYGON_RPC_URL=https://polygon-mainnet.infura.io/v3/your_infura_project_id
```

### 2. Start the System

```bash
# Start the complete system
./start.sh

# Or start components individually
./start.sh deps    # Start only dependencies
./start.sh build   # Build the Rust application
./start.sh app     # Start only the application
```

### 3. Verify Installation

```bash
# Check system status
./start.sh status

# Check API health
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8080/metrics
```

## 🌐 Access Points

Once started, you can access:

| Service | URL | Credentials |
|---------|-----|-------------|
| **API** | http://localhost:8080 | - |
| **Health Check** | http://localhost:8080/health | - |
| **Metrics** | http://localhost:8080/metrics | - |
| **Grafana** | http://localhost:3000 | admin/admin |
| **Prometheus** | http://localhost:9090 | - |
| **Jaeger** | http://localhost:16686 | - |
| **Kibana** | http://localhost:5601 | - |

## 🛠️ Configuration

### Environment Variables

The system uses environment variables for configuration. Key variables:

#### Security
- `JWT_SECRET` - JWT signing secret (32+ chars)
- `ENCRYPTION_KEY` - Data encryption key (32 chars)
- `RATE_LIMIT_REQUESTS` - API rate limit
- `ENABLE_AUDIT_LOGGING` - Enable audit logs

#### Database
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string

#### AI Providers
- `OPENAI_API_KEY` - OpenAI API key
- `ANTHROPIC_API_KEY` - Anthropic API key
- `GOOGLE_API_KEY` - Google AI API key

#### Monitoring
- `PROMETHEUS_ENDPOINT` - Prometheus metrics endpoint
- `JAEGER_ENDPOINT` - Jaeger tracing endpoint

### Configuration File

The system also supports YAML configuration via `rust-intelligence-system/config.template.yaml`:

```yaml
system:
  name: "AI Intelligence System"
  version: "1.0.0"
  environment: "production"
  debug: false

server:
  host: "0.0.0.0"
  port: 8080
  workers: 4
  max_connections: 1000

security:
  jwt_expiry_hours: 24
  rate_limit_requests: 1000
  enable_audit_logging: true
```

## 🔒 Security Setup

### 1. Generate Secure Keys

```bash
# Generate JWT secret (32+ characters)
openssl rand -base64 32

# Generate encryption key (32 characters)
openssl rand -base64 24
```

### 2. Database Security

```bash
# Use strong passwords
POSTGRES_PASSWORD=$(openssl rand -base64 32)
REDIS_PASSWORD=$(openssl rand -base64 32)
```

### 3. API Keys

Get API keys from:
- **OpenAI**: https://platform.openai.com/api-keys
- **Anthropic**: https://console.anthropic.com/
- **Google AI**: https://aistudio.google.com/app/apikey
- **Telegram**: https://my.telegram.org/apps

## 📊 Monitoring Setup

### 1. Grafana Dashboards

Access Grafana at http://localhost:3000 (admin/admin) to view:
- System metrics
- Application performance
- Database statistics
- API usage

### 2. Prometheus Metrics

Access Prometheus at http://localhost:9090 to view:
- Raw metrics data
- Query metrics
- Alert rules

### 3. Jaeger Tracing

Access Jaeger at http://localhost:16686 to view:
- Request traces
- Performance analysis
- Error tracking

## 🚨 Troubleshooting

### Common Issues

#### 1. Port Conflicts
```bash
# Check if ports are in use
netstat -tulpn | grep :8080
netstat -tulpn | grep :5432
netstat -tulpn | grep :6379
```

#### 2. Docker Issues
```bash
# Check Docker status
docker ps
docker-compose ps

# Restart Docker services
docker-compose restart
```

#### 3. Rust Build Issues
```bash
# Clean and rebuild
cd rust-intelligence-system
cargo clean
cargo build --release
```

#### 4. Database Connection Issues
```bash
# Check database status
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U intelligence -d intelligence -c "SELECT 1;"
```

### Logs

```bash
# Application logs
tail -f rust-intelligence-system/logs/app.log

# Docker logs
docker-compose logs -f

# System logs
journalctl -u intelligence-system
```

## 🔄 Maintenance

### Daily Operations

```bash
# Check system status
./start.sh status

# View logs
docker-compose logs -f intelligence-api

# Restart if needed
./stop.sh && ./start.sh
```

### Updates

```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
./stop.sh
./start.sh build
./start.sh app
```

### Backup

```bash
# Backup database
docker-compose exec postgres pg_dump -U intelligence intelligence > backup.sql

# Backup configuration
cp .env .env.backup
cp -r rust-intelligence-system/config/ config.backup/
```

## 🆘 Support

### Getting Help

1. **Check Logs**: Always check logs first
2. **System Status**: Run `./start.sh status`
3. **Health Checks**: Visit http://localhost:8080/health
4. **Documentation**: Check the README.md files

### Emergency Procedures

```bash
# Emergency stop
./stop.sh

# Force cleanup
docker-compose down -v
docker system prune -f

# Fresh start
./start.sh
```

## 📈 Performance Tuning

### Resource Optimization

```bash
# Increase Docker resources
# Edit docker-compose.yml to adjust:
# - memory limits
# - CPU limits
# - connection pools
```

### Database Tuning

```bash
# PostgreSQL tuning
# Edit postgresql.conf for:
# - shared_buffers
# - effective_cache_size
# - max_connections
```

## 🎯 Next Steps

After successful startup:

1. **Configure AI Providers**: Add your API keys
2. **Set Up Monitoring**: Customize Grafana dashboards
3. **Configure Data Sources**: Set up Telegram, blockchain connections
4. **Test APIs**: Use the API endpoints
5. **Set Up Alerts**: Configure monitoring alerts

---

**🎉 Congratulations!** Your AI Intelligence System is now running in production mode.
