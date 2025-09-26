#!/bin/bash

# HDD Tool Server Quick Setup Script for Ubuntu
# This script automates the Ubuntu server deployment process

set -e  # Exit on any error

echo "🚀 HDD Tool Server Setup - Ubuntu Deployment"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
DB_NAME="hdd_tool_db"
DB_USER="hdd_user"
DB_PASSWORD="$(openssl rand -base64 32)"
SERVER_PORT="3000"
DOMAIN=""

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root for security reasons"
   exit 1
fi

# Get domain name from user
read -p "Enter your domain name (or press Enter for IP-only setup): " DOMAIN

print_info "Step 1: Updating system packages..."
sudo apt update && sudo apt upgrade -y

print_info "Step 2: Installing essential packages..."
sudo apt install -y curl wget git build-essential pkg-config libssl-dev postgresql postgresql-contrib nginx ufw

print_info "Step 3: Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_status "Rust installed successfully"
else
    print_status "Rust already installed"
fi

print_info "Step 4: Setting up PostgreSQL..."
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE ${DB_NAME};
CREATE USER ${DB_USER} WITH ENCRYPTED PASSWORD '${DB_PASSWORD}';
GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO ${DB_USER};
ALTER USER ${DB_USER} CREATEDB;
\q
EOF

print_status "Database '${DB_NAME}' created with user '${DB_USER}'"

print_info "Step 5: Configuring PostgreSQL for network access..."
PG_VERSION=$(sudo -u postgres psql -t -c "SELECT version();" | grep -oP '\d+\.\d+' | head -1)
PG_CONFIG="/etc/postgresql/${PG_VERSION}/main/postgresql.conf"
PG_HBA="/etc/postgresql/${PG_VERSION}/main/pg_hba.conf"

# Backup original files
sudo cp "$PG_CONFIG" "${PG_CONFIG}.backup"
sudo cp "$PG_HBA" "${PG_HBA}.backup"

# Configure PostgreSQL
sudo sed -i "s/#listen_addresses = 'localhost'/listen_addresses = '*'/" "$PG_CONFIG"
echo "host    all             all             0.0.0.0/0               md5" | sudo tee -a "$PG_HBA"

sudo systemctl restart postgresql

print_info "Step 6: Setting up database schema..."
sudo -u postgres psql -d ${DB_NAME} << 'EOF'
-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true
);

-- Certificates table
CREATE TABLE IF NOT EXISTS certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    certificate_data TEXT NOT NULL,
    device_info JSONB NOT NULL,
    sanitization_method VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    file_hash VARCHAR(64) NOT NULL
);

-- Sanitization logs table
CREATE TABLE IF NOT EXISTS sanitization_logs (
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
    error_message TEXT
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_certificates_user_id ON certificates(user_id);
CREATE INDEX IF NOT EXISTS idx_certificates_created_at ON certificates(created_at);
CREATE INDEX IF NOT EXISTS idx_logs_user_id ON sanitization_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_logs_status ON sanitization_logs(status);
CREATE INDEX IF NOT EXISTS idx_logs_started_at ON sanitization_logs(started_at);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
EOF

print_status "Database schema created successfully"

print_info "Step 7: Cloning HDD Tool repository..."
if [ ! -d "HDD-Tool" ]; then
    git clone https://github.com/riteshvijaykumar/HDD-Tool.git
    cd HDD-Tool
else
    cd HDD-Tool
    git pull origin main
fi

print_info "Step 8: Building HDD Tool server..."
export DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@localhost/${DB_NAME}"
cargo build --bin hdd-tool-server --features server --release

print_status "Server built successfully"

print_info "Step 9: Creating systemd service..."
sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << EOF
[Unit]
Description=HDD Tool Certificate Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$(whoami)
WorkingDirectory=$(pwd)
Environment=DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@localhost/${DB_NAME}
Environment=RUST_LOG=info
Environment=PORT=${SERVER_PORT}
ExecStart=$(pwd)/target/release/hdd-tool-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable hdd-tool-server

print_info "Step 10: Configuring firewall..."
sudo ufw --force enable
sudo ufw allow ssh
sudo ufw allow ${SERVER_PORT}/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

if [ ! -z "$DOMAIN" ]; then
    print_info "Step 11: Setting up Nginx reverse proxy..."
    sudo tee /etc/nginx/sites-available/hdd-tool > /dev/null << EOF
server {
    listen 80;
    server_name ${DOMAIN};

    location / {
        proxy_pass http://127.0.0.1:${SERVER_PORT};
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

    sudo ln -sf /etc/nginx/sites-available/hdd-tool /etc/nginx/sites-enabled/
    sudo rm -f /etc/nginx/sites-enabled/default
    sudo nginx -t
    sudo systemctl restart nginx

    print_info "Step 12: Installing SSL certificate..."
    sudo apt install -y certbot python3-certbot-nginx
    
    print_warning "Run the following command after DNS is configured:"
    echo "sudo certbot --nginx -d ${DOMAIN}"
else
    print_warning "Skipping Nginx setup - no domain provided"
fi

print_info "Step 13: Starting services..."
sudo systemctl start hdd-tool-server
sudo systemctl status hdd-tool-server --no-pager

print_info "Step 14: Creating backup script..."
tee ~/backup-hdd-tool.sh > /dev/null << EOF
#!/bin/bash
# HDD Tool Database Backup Script
BACKUP_DIR="\$HOME/backups"
mkdir -p "\$BACKUP_DIR"
pg_dump -h localhost -U ${DB_USER} ${DB_NAME} > "\$BACKUP_DIR/hdd_tool_\$(date +%Y%m%d_%H%M%S).sql"
# Keep only last 30 days of backups
find "\$BACKUP_DIR" -name "hdd_tool_*.sql" -mtime +30 -delete
EOF

chmod +x ~/backup-hdd-tool.sh

# Add to crontab
(crontab -l 2>/dev/null; echo "0 2 * * * $HOME/backup-hdd-tool.sh") | crontab -

print_status "Backup script created and scheduled"

# Create environment file for desktop clients
tee desktop-client.env > /dev/null << EOF
# HDD Tool Desktop Client Configuration
# Copy this to your Windows machine and set the environment variable

HDD_TOOL_SERVER_URL=http://$(curl -s ifconfig.me):${SERVER_PORT}
EOF

if [ ! -z "$DOMAIN" ]; then
    tee desktop-client-ssl.env > /dev/null << EOF
# HDD Tool Desktop Client Configuration (SSL)
# Use this after SSL certificate is installed

HDD_TOOL_SERVER_URL=https://${DOMAIN}
EOF
fi

echo ""
echo "============================================="
print_status "🎉 HDD Tool Server Setup Complete!"
echo "============================================="
echo ""
print_info "Server Configuration:"
echo "  - Database: ${DB_NAME}"
echo "  - Database User: ${DB_USER}"
echo "  - Database Password: ${DB_PASSWORD}"
echo "  - Server Port: ${SERVER_PORT}"
echo "  - Server URL: http://$(curl -s ifconfig.me):${SERVER_PORT}"
if [ ! -z "$DOMAIN" ]; then
    echo "  - Domain: ${DOMAIN}"
    echo "  - SSL URL: https://${DOMAIN} (after SSL setup)"
fi
echo ""

print_info "Next Steps:"
echo "1. Test the server: curl http://$(curl -s ifconfig.me):${SERVER_PORT}/api/health"
echo "2. Access web dashboard: http://$(curl -s ifconfig.me):${SERVER_PORT}/dashboard"
if [ ! -z "$DOMAIN" ]; then
    echo "3. Configure DNS for ${DOMAIN} to point to this server"
    echo "4. Run: sudo certbot --nginx -d ${DOMAIN}"
fi
echo "5. Configure desktop clients with the environment variables in:"
echo "   - desktop-client.env (HTTP)"
if [ ! -z "$DOMAIN" ]; then
    echo "   - desktop-client-ssl.env (HTTPS)"
fi

echo ""
print_info "Useful Commands:"
echo "  - Check server status: sudo systemctl status hdd-tool-server"
echo "  - View server logs: sudo journalctl -u hdd-tool-server -f"
echo "  - Restart server: sudo systemctl restart hdd-tool-server"
echo "  - Backup database: ~/backup-hdd-tool.sh"
echo "  - Connect to database: psql -h localhost -U ${DB_USER} -d ${DB_NAME}"

echo ""
print_warning "IMPORTANT: Save the database password: ${DB_PASSWORD}"
print_warning "Store this in a secure location!"

echo ""
print_status "Setup completed successfully! 🚀"