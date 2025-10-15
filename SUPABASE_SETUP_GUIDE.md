# 🚀 SUPABASE AI Intelligence System - Setup Guide

This guide will help you integrate Supabase with your AI Intelligence System for enhanced database, authentication, storage, and real-time features.

## 💰 Cost Breakdown: FREE with Supabase

| Component | Cost | Technology |
|-----------|------|------------|
| **Database** | $0 | Supabase PostgreSQL (500MB free) |
| **Authentication** | $0 | Supabase Auth (50k users free) |
| **Storage** | $0 | Supabase Storage (1GB free) |
| **Realtime** | $0 | Supabase Realtime (unlimited) |
| **AI Models** | $0 | Ollama (local) |
| **Monitoring** | $0 | Prometheus/Grafana (local) |
| **Total** | **$0.00** | **100% Free with Supabase** |

## 🚀 Quick Start with Supabase

### Prerequisites
- **Rust 1.75+** - [Install Rust](https://rustup.rs/) (FREE)
- **Docker & Docker Compose** - [Install Docker](https://docs.docker.com/get-docker/) (FREE)
- **Supabase Account** - [Sign up at Supabase](https://supabase.com) (FREE)
- **4GB RAM minimum** - Uses your existing hardware

### 1. Create Supabase Project

```bash
# 1. Go to https://supabase.com and sign up
# 2. Create a new project
# 3. Choose a region close to you
# 4. Set a strong database password
# 5. Wait for project to be ready (2-3 minutes)
```

### 2. Get Supabase Credentials

```bash
# In your Supabase dashboard:
# 1. Go to Settings > API
# 2. Copy your Project URL
# 3. Copy your anon/public key
# 4. Copy your service_role key (keep this secret!)
# 5. Go to Settings > Database
# 6. Copy your database connection string
```

### 3. Environment Setup

```bash
# Copy the Supabase environment template
cp supabase.env .env

# Edit with your actual Supabase credentials
nano .env
```

**Required Supabase Configuration:**
```bash
# Supabase Project Settings
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_ANON_KEY=your_supabase_anon_key_here
SUPABASE_SERVICE_ROLE_KEY=your_supabase_service_role_key_here

# Database Connection
DATABASE_URL=postgresql://postgres:your_password@db.your-project-id.supabase.co:5432/postgres
```

### 4. Start the Supabase System

```bash
# Make scripts executable
chmod +x start-supabase.sh stop-supabase.sh

# Validate configuration
./start-supabase.sh validate

# Start complete system
./start-supabase.sh
```

### 5. Access Services

| Service | URL | Purpose |
|---------|-----|---------|
| **API** | http://localhost:8080 | Main application |
| **Supabase Dashboard** | https://supabase.com/dashboard | Database management |
| **Ollama AI** | http://localhost:11434 | Free local AI models |
| **Grafana** | http://localhost:3000 | Monitoring (admin/admin) |
| **Prometheus** | http://localhost:9090 | Metrics |

## 🗄️ Supabase Database Setup

### 1. Create Tables

In your Supabase SQL Editor, run:

```sql
-- Users table
CREATE TABLE users (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Intelligence data table
CREATE TABLE intelligence_data (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    data_type TEXT NOT NULL,
    content JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Analytics results table
CREATE TABLE analytics_results (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    analysis_type TEXT NOT NULL,
    results JSONB NOT NULL,
    confidence_score FLOAT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- System logs table
CREATE TABLE system_logs (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Enable Row Level Security
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE intelligence_data ENABLE ROW LEVEL SECURITY;
ALTER TABLE analytics_results ENABLE ROW LEVEL SECURITY;

-- Create RLS policies
CREATE POLICY "Users can view own data" ON users
    FOR SELECT USING (auth.uid() = id);

CREATE POLICY "Users can insert own data" ON users
    FOR INSERT WITH CHECK (auth.uid() = id);

CREATE POLICY "Users can view own intelligence data" ON intelligence_data
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own intelligence data" ON intelligence_data
    FOR INSERT WITH CHECK (auth.uid() = user_id);
```

### 2. Enable Realtime

```sql
-- Enable realtime for tables
ALTER PUBLICATION supabase_realtime ADD TABLE intelligence_data;
ALTER PUBLICATION supabase_realtime ADD TABLE analytics_results;
ALTER PUBLICATION supabase_realtime ADD TABLE system_logs;
```

### 3. Create Storage Buckets

In Supabase Dashboard > Storage:

```bash
# Create buckets
1. intelligence-public (public access)
2. intelligence-private (authenticated access)
3. intelligence-models (authenticated access)
```

## 🔐 Supabase Authentication Setup

### 1. Configure Auth Providers

In Supabase Dashboard > Authentication > Providers:

```bash
# Enable providers
- Email (enabled by default)
- GitHub (optional)
- Google (optional)
- Discord (optional)
```

### 2. Configure Auth Settings

```bash
# Site URL: http://localhost:8080
# Redirect URLs: http://localhost:8080/auth/callback
# JWT expiry: 3600 seconds (1 hour)
# Enable email confirmations: true
```

### 3. Test Authentication

```bash
# Test sign up
curl -X POST 'https://your-project-id.supabase.co/auth/v1/signup' \
  -H "apikey: your-anon-key" \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'

# Test sign in
curl -X POST 'https://your-project-id.supabase.co/auth/v1/token?grant_type=password' \
  -H "apikey: your-anon-key" \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'
```

## 📊 Supabase Features

### ✅ Included Features

#### **Database (PostgreSQL)**
- **500MB storage** (free tier)
- **Full SQL support**
- **Real-time subscriptions**
- **Row Level Security (RLS)**
- **Database backups**

#### **Authentication**
- **50,000 monthly active users** (free tier)
- **Email/password authentication**
- **Social providers** (GitHub, Google, etc.)
- **JWT tokens**
- **User management**

#### **Storage**
- **1GB file storage** (free tier)
- **Public and private buckets**
- **CDN delivery**
- **Image transformations**

#### **Realtime**
- **Unlimited connections** (free tier)
- **Real-time subscriptions**
- **Presence tracking**
- **Broadcast channels**

### 🔧 Configuration

#### **Environment Variables**
```bash
# Supabase Configuration
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_ANON_KEY=your_anon_key
SUPABASE_SERVICE_ROLE_KEY=your_service_role_key

# Database
DATABASE_URL=postgresql://postgres:password@db.project.supabase.co:5432/postgres

# Auth
SUPABASE_AUTH_ENABLED=true
SUPABASE_JWT_SECRET=your_jwt_secret

# Storage
SUPABASE_STORAGE_ENABLED=true
SUPABASE_STORAGE_BUCKET=intelligence-data

# Realtime
SUPABASE_REALTIME_ENABLED=true
```

#### **Rust Integration**
```rust
// Add to Cargo.toml
[dependencies]
supabase = "0.3.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

// Initialize Supabase client
use supabase::Client;

let client = Client::new(
    "https://your-project-id.supabase.co",
    "your-anon-key"
)?;

// Database operations
let data: Vec<serde_json::Value> = client
    .database()
    .from("intelligence_data")
    .select("*")
    .execute()
    .await?;

// Authentication
let auth_response = client.auth()
    .sign_up_with_email_and_password("user@example.com", "password")
    .await?;

// Storage
client.storage()
    .from("intelligence-public")
    .upload("file.txt", file_data)
    .await?;

// Realtime
let mut subscription = client.realtime()
    .from("intelligence_data")
    .on("INSERT", |payload| {
        println!("New data: {:?}", payload);
    })
    .subscribe()
    .await?;
```

## 🛠️ Supabase Commands

```bash
# Start Supabase system
./start-supabase.sh

# Stop Supabase system
./stop-supabase.sh

# Validate configuration
./start-supabase.sh validate

# Check status
./start-supabase.sh status

# Start only dependencies
./start-supabase.sh deps

# Install Ollama
./start-supabase.sh ollama

# Start monitoring
./start-supabase.sh monitoring
```

## 📈 Supabase Performance

### **Free Tier Limits**
- **Database**: 500MB storage
- **Storage**: 1GB file storage
- **Auth**: 50,000 monthly active users
- **Realtime**: Unlimited connections
- **Edge Functions**: 2M invocations/month

### **Performance Characteristics**
- **Database Latency**: <50ms (global CDN)
- **Auth Response**: <100ms
- **Storage Upload**: Fast CDN delivery
- **Realtime**: <10ms latency
- **API Rate Limit**: 100 requests/minute

## 🔒 Supabase Security

### **Built-in Security**
- **Row Level Security (RLS)** - Database-level access control
- **JWT Authentication** - Secure token-based auth
- **API Key Management** - Separate keys for different access levels
- **SSL/TLS** - All connections encrypted
- **GDPR Compliant** - Data protection compliance

### **Security Best Practices**
```sql
-- Enable RLS on all tables
ALTER TABLE your_table ENABLE ROW LEVEL SECURITY;

-- Create policies
CREATE POLICY "policy_name" ON your_table
    FOR SELECT USING (auth.uid() = user_id);

-- Use service role key only server-side
-- Use anon key for client-side operations
```

## 🚨 Troubleshooting

### **Common Issues**

#### 1. Connection Errors
```bash
# Check Supabase URL format
SUPABASE_URL=https://your-project-id.supabase.co

# Verify API keys
curl -H "apikey: your-anon-key" https://your-project-id.supabase.co/rest/v1/
```

#### 2. Authentication Issues
```bash
# Check auth configuration
# Verify redirect URLs in Supabase dashboard
# Ensure JWT secret matches
```

#### 3. Database Connection
```bash
# Test database connection
psql "postgresql://postgres:password@db.project.supabase.co:5432/postgres"

# Check RLS policies
SELECT * FROM pg_policies WHERE tablename = 'your_table';
```

#### 4. Storage Issues
```bash
# Check bucket permissions
# Verify storage policies
# Test file upload
```

### **Logs and Debugging**
```bash
# Application logs
tail -f rust-intelligence-system/logs/app.log

# Docker logs
docker-compose -f rust-intelligence-system/docker-compose.supabase.yml logs -f

# Supabase logs (in dashboard)
# Go to Logs section in Supabase dashboard
```

## 🔄 Maintenance

### **Daily Operations**
```bash
# Check system status
./start-supabase.sh status

# Monitor Supabase usage
# Check dashboard for usage metrics

# Backup data
# Supabase handles automatic backups
```

### **Updates**
```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
./stop-supabase.sh
./start-supabase.sh build
./start-supabase.sh app
```

### **Monitoring**
- **Supabase Dashboard** - Usage metrics, logs, performance
- **Grafana** - Application metrics
- **Prometheus** - System metrics
- **Database** - Query performance, connection stats

## 🎯 Use Cases

### **Perfect For**
- **Web Applications** - Full-stack apps with auth
- **Real-time Apps** - Live data updates
- **File Storage** - Document and media storage
- **Multi-user Systems** - User management and permissions
- **API Development** - RESTful APIs with database

### **Not Suitable For**
- **High-volume Analytics** - Complex data processing
- **Machine Learning** - Large-scale ML workloads
- **Video Processing** - Heavy media processing
- **Blockchain Mining** - Resource-intensive operations

## 🚀 Upgrade Path

### **When You Need More**
1. **Pro Plan** - $25/month for more resources
2. **Team Plan** - $599/month for teams
3. **Enterprise** - Custom pricing for large scale

### **Migration Steps**
```bash
# 1. Export data from free tier
pg_dump "postgresql://postgres:password@db.project.supabase.co:5432/postgres" > backup.sql

# 2. Upgrade Supabase plan
# 3. Import data to new instance
# 4. Update environment variables
# 5. Test all functionality
```

## 📚 Resources

### **Documentation**
- **[Supabase Docs](https://supabase.com/docs)** - Official documentation
- **[Supabase Rust Client](https://docs.rs/supabase-lib-rs/)** - Rust integration
- **[PostgreSQL Docs](https://www.postgresql.org/docs/)** - Database reference

### **Community**
- **[Supabase Discord](https://discord.supabase.com)** - Community support
- **[GitHub Discussions](https://github.com/supabase/supabase/discussions)** - Technical discussions
- **[Stack Overflow](https://stackoverflow.com/questions/tagged/supabase)** - Q&A

## 🎉 Success!

You now have a **Supabase-powered** AI Intelligence System with:

- ✅ **Free PostgreSQL database** (500MB)
- ✅ **Authentication system** (50k users)
- ✅ **File storage** (1GB)
- ✅ **Real-time subscriptions**
- ✅ **Local AI models** (Ollama)
- ✅ **Monitoring and analytics**
- ✅ **Row-level security**

**Total Cost: $0.00** with Supabase free tier! 🎉

---

**Need help?** Check the troubleshooting section or visit the Supabase community forums.
