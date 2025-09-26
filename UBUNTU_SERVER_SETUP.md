# Ubuntu Server Deployment Guide

This guide will help you deploy the HDD Tool server on Ubuntu with PostgreSQL database and secure APIs.

## 🖥️ Ubuntu Server Setup

### 1. Initial Server Preparation

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install essential packages
sudo apt install -y curl wget git build-essential pkg-config libssl-dev
```

### 2. Install Rust on Ubuntu

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 3. Install PostgreSQL

```bash
# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Start and enable PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE hdd_tool_db;
CREATE USER hdd_user WITH ENCRYPTED PASSWORD 'secure_password_here';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;
ALTER USER hdd_user CREATEDB;
\q
EOF
```

### 4. Configure PostgreSQL for Remote Access

```bash
# Edit PostgreSQL configuration
sudo nano /etc/postgresql/*/main/postgresql.conf

# Find and modify this line:
# listen_addresses = 'localhost'
# Change to:
listen_addresses = '*'

# Edit host-based authentication
sudo nano /etc/postgresql/*/main/pg_hba.conf

# Add this line for your network (adjust IP range as needed):
host    all             all             0.0.0.0/0               md5

# Restart PostgreSQL
sudo systemctl restart postgresql
```

### 5. Setup Database Schema

```bash
# Connect to database and create schema
sudo -u postgres psql -d hdd_tool_db << 'EOF'
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true
);

-- Certificates table
CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    certificate_data TEXT NOT NULL,
    device_info JSONB NOT NULL,
    sanitization_method VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    file_hash VARCHAR(64) NOT NULL,
    
    INDEX idx_certificates_user_id (user_id),
    INDEX idx_certificates_created_at (created_at)
);

-- Sanitization logs table
CREATE TABLE sanitization_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    certificate_id UUID REFERENCES certificates(id) ON DELETE SET NULL,
    device_path VARCHAR(255) NOT NULL,
    device_type VARCHAR(50) NOT NULL,
    method VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    sectors_processed BIGINT,
    error_message TEXT,
    
    INDEX idx_logs_user_id (user_id),
    INDEX idx_logs_status (status),
    INDEX idx_logs_started_at (started_at)
);

-- Create indexes for better performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
EOF
```

### 6. Clone and Build HDD Tool Server

```bash
# Clone the repository
git clone https://github.com/riteshvijaykumar/HDD-Tool.git
cd HDD-Tool

# Set environment variables
export DATABASE_URL="postgresql://hdd_user:secure_password_here@localhost/hdd_tool_db"

# Build the server with features
cargo build --bin hdd-tool-server --features server --release
```

### 7. Configure Firewall

```bash
# Install and configure ufw firewall
sudo ufw enable
sudo ufw allow ssh
sudo ufw allow 3000/tcp    # HDD Tool API server
sudo ufw allow 5432/tcp   # PostgreSQL (if accessing externally)
sudo ufw status
```

### 8. Create Systemd Service

```bash
# Create service file
sudo nano /etc/systemd/system/hdd-tool-server.service
```

Add this content:

```ini
[Unit]
Description=HDD Tool Certificate Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/HDD-Tool
Environment=DATABASE_URL=postgresql://hdd_user:secure_password_here@localhost/hdd_tool_db
Environment=RUST_LOG=info
ExecStart=/home/ubuntu/HDD-Tool/target/release/hdd-tool-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable hdd-tool-server
sudo systemctl start hdd-tool-server

# Check status
sudo systemctl status hdd-tool-server
```

### 9. Setup SSL/TLS (Recommended for Production)

```bash
# Install Nginx for reverse proxy
sudo apt install -y nginx

# Install Certbot for Let's Encrypt SSL
sudo apt install -y certbot python3-certbot-nginx

# Configure Nginx
sudo nano /etc/nginx/sites-available/hdd-tool
```

Add this Nginx configuration:

```nginx
server {
    listen 80;
    server_name your-domain.com;  # Replace with your domain

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

```bash
# Enable the site
sudo ln -s /etc/nginx/sites-available/hdd-tool /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx

# Get SSL certificate
sudo certbot --nginx -d your-domain.com
```

## 🔧 Client Configuration (Windows Desktop)

### Update Desktop Client for Remote Server

The desktop application needs to be configured to connect to your Ubuntu server instead of localhost.

### Configuration Options

You can configure the server URL in several ways:

1. **Environment Variable** (Recommended):
```powershell
# On Windows, set environment variable
$env:HDD_TOOL_SERVER_URL = "https://your-domain.com"
# or for HTTP (not recommended for production)
$env:HDD_TOOL_SERVER_URL = "http://your-server-ip:3000"
```

2. **Configuration File**: Create `config.json` in the application directory:
```json
{
    "server_url": "https://your-domain.com",
    "enable_server_sync": true,
    "auto_upload_certificates": true
}
```

## 🚀 Testing the Deployment

### 1. Test Server Endpoints

```bash
# Test server health
curl http://your-server-ip:3000/api/health

# Test user registration
curl -X POST http://your-server-ip:3000/api/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"testpass123"}'

# Test login
curl -X POST http://your-server-ip:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass123"}'
```

### 2. Test Database Connection

```bash
# Connect to database
sudo -u postgres psql -d hdd_tool_db

# Check tables
\dt

# Check if user was created
SELECT * FROM users;
```

### 3. Monitor Logs

```bash
# Check server logs
sudo journalctl -u hdd-tool-server -f

# Check Nginx logs (if using SSL)
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log
```

## 🔐 Security Best Practices

### 1. Database Security
- Use strong passwords for PostgreSQL users
- Limit network access to database
- Enable SSL for database connections
- Regular backups

### 2. Server Security
- Keep Ubuntu system updated
- Use firewall (ufw) to limit port access
- Enable SSL/TLS for API endpoints
- Use strong authentication tokens
- Regular security updates

### 3. Network Security
- Use HTTPS for all API communication
- Implement rate limiting
- Monitor failed authentication attempts
- Use VPN for admin access

## 📊 Monitoring and Maintenance

### 1. Log Management
```bash
# Rotate logs
sudo nano /etc/logrotate.d/hdd-tool-server

# Add log rotation configuration
/var/log/hdd-tool-server.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0644 ubuntu ubuntu
}
```

### 2. Database Maintenance
```bash
# Create backup script
nano ~/backup-db.sh

#!/bin/bash
pg_dump -h localhost -U hdd_user hdd_tool_db > /backups/hdd_tool_$(date +%Y%m%d).sql

# Make executable and add to cron
chmod +x ~/backup-db.sh
crontab -e
# Add: 0 2 * * * /home/ubuntu/backup-db.sh
```

### 3. Performance Monitoring
```bash
# Install monitoring tools
sudo apt install -y htop iotop nethogs

# Monitor server resources
htop
sudo iotop
sudo nethogs
```

## 🌐 API Endpoints

Your Ubuntu server will provide these secure API endpoints:

- `POST /api/register` - User registration
- `POST /api/login` - User authentication
- `POST /api/certificates` - Submit certificate
- `GET /api/certificates` - Get user certificates
- `GET /api/logs` - Get sanitization logs
- `GET /dashboard` - Web dashboard

## 📱 Desktop Client Usage

Once the server is deployed:

1. Launch the Windows desktop application
2. Configure server URL (environment variable or config file)
3. Register/login with server credentials
4. Perform disk sanitization
5. Certificates automatically sync to Ubuntu server
6. View history via web dashboard at `https://your-domain.com/dashboard`

This architecture provides:
- ✅ Secure Ubuntu server deployment
- ✅ PostgreSQL database with proper schema
- ✅ SSL/TLS encrypted API communication
- ✅ Professional web dashboard
- ✅ Automatic certificate synchronization
- ✅ Centralized user management
- ✅ Complete audit trail