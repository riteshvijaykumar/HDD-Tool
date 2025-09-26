#!/bin/bash

# Quick deployment script for HDD Tool
# Supports multiple deployment targets

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Usage information
usage() {
    echo "HDD Tool Deployment Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  local           Install locally for development"
    echo "  build           Build release packages"
    echo "  server          Deploy server components"
    echo "  docker          Build and run Docker containers"
    echo "  clean           Clean build artifacts"
    echo ""
    echo "Options:"
    echo "  --version VERSION   Specify version (default: v0.1.0)"
    echo "  --target TARGET     Specify build target"
    echo "  --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 local                    # Install for local development"
    echo "  $0 build --version v1.0.0   # Build release v1.0.0"
    echo "  $0 server                   # Deploy server components"
    echo "  $0 docker                   # Build Docker containers"
}

# Local development installation
install_local() {
    print_status "Installing HDD Tool for local development..."
    
    cd "$PROJECT_ROOT"
    
    # Build desktop application
    cargo build --release --bin hdd-tool
    
    # Build server application
    cargo build --release --bin hdd-tool-server --features server
    
    # Create local bin directory
    mkdir -p ~/.local/bin
    
    # Copy binaries
    cp target/release/hdd-tool ~/.local/bin/
    cp target/release/hdd-tool-server ~/.local/bin/
    
    # Make executable
    chmod +x ~/.local/bin/hdd-tool
    chmod +x ~/.local/bin/hdd-tool-server
    
    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
        print_warning "Added ~/.local/bin to PATH. Please restart your shell or run: source ~/.bashrc"
    fi
    
    print_success "Local installation completed!"
    print_status "You can now run: hdd-tool"
}

# Build release packages
build_release() {
    local version=${1:-"v0.1.0"}
    
    print_status "Building release packages for version: $version"
    
    cd "$PROJECT_ROOT"
    
    # Run build script
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        # Windows
        cmd //c "scripts\\build-release.bat $version"
    else
        # Unix-like systems
        chmod +x scripts/build-release.sh
        ./scripts/build-release.sh "$version"
    fi
}

# Deploy server components
deploy_server() {
    print_status "Deploying HDD Tool server components..."
    
    cd "$PROJECT_ROOT"
    
    # Check if PostgreSQL is available
    if ! command -v psql &> /dev/null; then
        print_error "PostgreSQL is not installed. Please install PostgreSQL first."
        exit 1
    fi
    
    # Build server
    cargo build --release --bin hdd-tool-server --features server
    
    # Create server directory
    sudo mkdir -p /opt/hdd-tool-server
    sudo cp target/release/hdd-tool-server /opt/hdd-tool-server/
    sudo chmod +x /opt/hdd-tool-server/hdd-tool-server
    
    # Copy web assets
    if [ -d "web" ]; then
        sudo cp -r web /opt/hdd-tool-server/
    fi
    
    # Create systemd service
    sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << EOF
[Unit]
Description=HDD Tool Server
After=network.target postgresql.service

[Service]
Type=simple
User=hdd-tool
WorkingDirectory=/opt/hdd-tool-server
ExecStart=/opt/hdd-tool-server/hdd-tool-server
Restart=always
RestartSec=10
Environment=DATABASE_URL=postgresql://hdd_tool:password@localhost/hdd_tool_db
Environment=SERVER_PORT=3030
Environment=SERVER_HOST=0.0.0.0

[Install]
WantedBy=multi-user.target
EOF
    
    # Create user
    if ! id "hdd-tool" &>/dev/null; then
        sudo useradd -r -s /bin/false -d /opt/hdd-tool-server hdd-tool
    fi
    
    sudo chown -R hdd-tool:hdd-tool /opt/hdd-tool-server
    
    # Reload systemd
    sudo systemctl daemon-reload
    
    print_success "Server deployment completed!"
    print_status "Start the server with: sudo systemctl start hdd-tool-server"
    print_status "Enable auto-start with: sudo systemctl enable hdd-tool-server"
}

# Docker deployment
deploy_docker() {
    print_status "Building Docker containers..."
    
    cd "$PROJECT_ROOT"
    
    # Create Dockerfile if it doesn't exist
    if [ ! -f "Dockerfile" ]; then
        cat > Dockerfile << 'EOF'
# Multi-stage build for HDD Tool
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build the server application
RUN cargo build --release --bin hdd-tool-server --features server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false hdd-tool

# Copy binary
COPY --from=builder /app/target/release/hdd-tool-server /usr/local/bin/
COPY --from=builder /app/web /opt/hdd-tool/web

# Set ownership
RUN chown -R hdd-tool:hdd-tool /opt/hdd-tool

USER hdd-tool
WORKDIR /opt/hdd-tool

EXPOSE 3030

CMD ["hdd-tool-server"]
EOF
    fi
    
    # Create docker-compose.yml
    if [ ! -f "docker-compose.yml" ]; then
        cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  hdd-tool-server:
    build: .
    ports:
      - "3030:3030"
    environment:
      - DATABASE_URL=postgresql://hdd_tool:password@postgres:5432/hdd_tool_db
      - SERVER_PORT=3030
      - SERVER_HOST=0.0.0.0
    depends_on:
      - postgres
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=hdd_tool_db
      - POSTGRES_USER=hdd_tool
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data:
EOF
    fi
    
    # Build and start containers
    docker-compose build
    docker-compose up -d
    
    print_success "Docker deployment completed!"
    print_status "Access the web interface at: http://localhost:3030"
}

# Clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    
    cd "$PROJECT_ROOT"
    
    # Clean Cargo build
    cargo clean
    
    # Remove release directories
    rm -rf releases/
    rm -rf dist/packages/
    
    # Remove Docker containers and images
    if command -v docker &> /dev/null; then
        docker-compose down --volumes --remove-orphans 2>/dev/null || true
        docker system prune -f 2>/dev/null || true
    fi
    
    print_success "Clean completed!"
}

# Main function
main() {
    local command="$1"
    shift || true
    
    case "$command" in
        "local")
            install_local "$@"
            ;;
        "build")
            local version="v0.1.0"
            while [[ $# -gt 0 ]]; do
                case $1 in
                    --version)
                        version="$2"
                        shift 2
                        ;;
                    *)
                        shift
                        ;;
                esac
            done
            build_release "$version"
            ;;
        "server")
            deploy_server "$@"
            ;;
        "docker")
            deploy_docker "$@"
            ;;
        "clean")
            clean_build "$@"
            ;;
        "help"|"--help"|"-h")
            usage
            ;;
        "")
            print_error "No command specified"
            usage
            exit 1
            ;;
        *)
            print_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"