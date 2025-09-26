# 🚀 HDD Tool Server Setup Guide

## 📋 Overview

The HDD Tool Server provides a centralized system for:
- **User Account Management** - Create and authenticate user accounts
- **Certificate Logging** - Store and track sanitization certificates  
- **Activity Monitoring** - Log all sanitization activities per user
- **Web Dashboard** - View certificates and logs through a web interface

## 🏗️ Architecture

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Desktop Client    │    │    Web Server       │    │    PostgreSQL       │
│                     │    │                     │    │                     │
│  ┌─────────────────┐│    │  ┌─────────────────┐│    │  ┌─────────────────┐│
│  │ HDD Tool App    ││◄──►│  │ REST API        ││◄──►│  │ Users           ││
│  │ - Local Auth    ││    │  │ - User Auth     ││    │  │ Certificates    ││
│  │ - Server Sync   ││    │  │ - Cert Storage  ││    │  │ Sanitize Logs   ││
│  └─────────────────┘│    │  │ - Activity Log  ││    │  └─────────────────┘│
│                     │    │  └─────────────────┘│    │                     │
│  User Credentials   │    │                     │    │    Database         │
│  Certificate Upload │    │  Web Dashboard      │    │                     │
└─────────────────────┘    │  Static Files       │    └─────────────────────┘
                           └─────────────────────┘
```

## 🛠️ Prerequisites

### 1. PostgreSQL Database
```bash
# Install PostgreSQL (Ubuntu/Debian)
sudo apt update
sudo apt install postgresql postgresql-contrib

# Install PostgreSQL (Windows)
# Download from: https://www.postgresql.org/download/windows/

# Install PostgreSQL (macOS)
brew install postgresql
```

### 2. Create Database
```sql
-- Connect to PostgreSQL as superuser
sudo -u postgres psql

-- Create database and user
CREATE DATABASE hdd_tool;
CREATE USER hdd_user WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool TO hdd_user;

-- Exit PostgreSQL
\q
```

## 📦 Installation

### 1. Build the Server
```bash
# Clone the repository
git clone https://github.com/riteshvijaykumar/HDD-Tool.git
cd HDD-Tool

# Build the server binary
cargo build --release --bin hdd-tool-server

# The server binary will be at: target/release/hdd-tool-server
```

### 2. Environment Configuration
```bash
# Create environment file
cat > .env << EOF
DATABASE_URL=postgresql://hdd_user:secure_password@localhost/hdd_tool
PORT=8080
RUST_LOG=info
EOF

# Export environment variables
export DATABASE_URL="postgresql://hdd_user:secure_password@localhost/hdd_tool"
export PORT=8080
```

## 🚀 Running the Server

### 1. Start the Server
```bash
# Using environment variables  
export DATABASE_URL="postgresql://hdd_user:secure_password@localhost/hdd_tool"
./target/release/hdd-tool-server

# Or using environment file
source .env
./target/release/hdd-tool-server
```

### 2. Server Output
```
🗄️  Database URL: postgresql://hdd_user:secure_password@localhost/hdd_tool  
🚀 HDD Tool Server starting on port 8080
📊 Dashboard available at: http://localhost:8080/dashboard
🔗 API endpoints:
   POST /api/register - Create user account
   POST /api/login - User login  
   POST /api/certificates - Submit certificate
   GET  /api/certificates - Get user certificates
   GET  /api/logs - Get sanitization logs
```

## 🌐 Web Dashboard

### Access the Dashboard
1. Open browser: **http://localhost:8080/dashboard**
2. **Register** a new account or **Login** with existing credentials
3. View your **certificates** and **sanitization logs**

### Features
- **📋 Certificate Management** - View all your sanitization certificates
- **📊 Activity Logs** - Track sanitization operations with status and timing
- **🔐 User Authentication** - Secure login with password hashing
- **📱 Responsive Design** - Works on desktop and mobile devices

## 🔧 Desktop Client Integration

### 1. Enable Server Sync in Desktop App
```rust
// The desktop app will automatically connect to the server
// when server_enabled is set to true in the settings
```

### 2. User Workflow
1. **Create Account** - Register on web dashboard or through desktop app
2. **Login** - Authenticate in desktop application  
3. **Sanitize Devices** - Perform sanitization operations
4. **Auto-Upload** - Certificates automatically uploaded to server
5. **View Online** - Access certificates and logs through web dashboard

## 📊 API Endpoints

### Authentication
```bash
# Register new user
curl -X POST http://localhost:8080/api/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "user@example.com", 
    "password": "secure123"
  }'

# Login user
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "secure123"
  }'
```

### Certificate Management
```bash
# Submit certificate (requires Bearer token)
curl -X POST http://localhost:8080/api/certificates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "certificate_data": "{\"device\":\"sda\",\"method\":\"NIST Clear\"}",
    "device_info": "/dev/sda - Samsung SSD 1TB",
    "sanitization_method": "NIST SP 800-88 Clear"
  }'

# Get user certificates
curl -X GET "http://localhost:8080/api/certificates?limit=10&offset=0" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Get sanitization logs  
curl -X GET "http://localhost:8080/api/logs?limit=10&offset=0" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## 🗄️ Database Schema

### Tables Created Automatically
```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL, 
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE
);

-- Certificates table  
CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    certificate_data TEXT NOT NULL,
    device_info VARCHAR(500) NOT NULL,
    sanitization_method VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    file_hash VARCHAR(255) NOT NULL
);

-- Sanitization logs table
CREATE TABLE sanitization_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    certificate_id UUID REFERENCES certificates(id),
    device_path VARCHAR(500) NOT NULL,
    device_type VARCHAR(100) NOT NULL,
    method VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    duration_seconds BIGINT,
    bytes_processed BIGINT,
    error_message TEXT,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
```

## 🔐 Security Features

### 1. Password Security
- **SHA-256 Hashing** - Passwords are never stored in plain text
- **Salt-based Hashing** - Each password uses unique salt  
- **Secure Transmission** - HTTPS recommended for production

### 2. Authentication
- **Bearer Tokens** - Simple token-based authentication
- **User Isolation** - Users can only access their own data
- **Session Management** - Tokens can be invalidated

### 3. Data Integrity  
- **File Hashing** - Certificate integrity verification
- **Database Constraints** - Foreign key relationships enforced
- **Input Validation** - All API inputs validated

## 🚀 Production Deployment

### 1. Environment Variables
```bash
# Production configuration
export DATABASE_URL="postgresql://user:pass@db-server:5432/hdd_tool"
export PORT=8080
export RUST_LOG=warn
```

### 2. Systemd Service (Linux)
```ini
# /etc/systemd/system/hdd-tool-server.service
[Unit]
Description=HDD Tool Server
After=network.target

[Service]
Type=simple
User=hdd-tool
WorkingDirectory=/opt/hdd-tool
ExecStart=/opt/hdd-tool/hdd-tool-server
Environment=DATABASE_URL=postgresql://user:pass@localhost/hdd_tool
Environment=PORT=8080
Restart=always

[Install]
WantedBy=multi-user.target
```

### 3. Reverse Proxy (Nginx)
```nginx
# /etc/nginx/sites-available/hdd-tool
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 📈 Monitoring & Logs

### 1. Application Logs
```bash
# View server logs
RUST_LOG=info ./hdd-tool-server

# Log levels: error, warn, info, debug, trace
```

### 2. Database Monitoring
```sql
-- Check active connections
SELECT * FROM pg_stat_activity WHERE datname = 'hdd_tool';

-- Check table sizes
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public';
```

## 🔧 Troubleshooting

### Common Issues

1. **Database Connection Failed**
   ```bash
   # Check PostgreSQL is running
   sudo systemctl status postgresql
   
   # Test connection manually  
   psql -h localhost -U hdd_user -d hdd_tool
   ```

2. **Port Already in Use**
   ```bash
   # Check what's using port 8080
   sudo netstat -tulpn | grep :8080
   
   # Use different port
   export PORT=8081
   ```

3. **Permission Denied**
   ```bash
   # Check file permissions
   ls -la target/release/hdd-tool-server
   
   # Make executable
   chmod +x target/release/hdd-tool-server
   ```

## 📝 Development

### 1. Running in Development
```bash
# Run with auto-reload
cargo watch -x "run --bin hdd-tool-server"

# Run with debug logging
RUST_LOG=debug cargo run --bin hdd-tool-server
```

### 2. Database Migrations
```bash
# Reset database (development only)
psql -h localhost -U hdd_user -d hdd_tool -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
```

---

## 🎯 Next Steps

1. **Deploy Server** - Set up on cloud or local server
2. **Configure Desktop App** - Enable server sync in client
3. **Create User Accounts** - Register users through web dashboard  
4. **Monitor Usage** - Track certificates and sanitization activities
5. **Scale as Needed** - Add load balancing and database replication

Your HDD Tool now has enterprise-grade server capabilities! 🚀