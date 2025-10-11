#!/bin/bash

# lock-rs installer
# Yet another paranoid lock - zeroidize your personal pixels before they hit the cloud

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="elly-hacen/lock-rs"
BINARY_NAME="lock"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TEMP_DIR=$(mktemp -d)

# Cleanup function
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          print_error "Unsupported operating system: $(uname -s)"; exit 1 ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        armv7l) arch="armv7" ;;
        *) print_error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
    
    echo "${os}-${arch}"
}

# Get latest release version
get_latest_version() {
    local version
    version=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$version" ]; then
        print_error "Failed to get latest version"
        exit 1
    fi
    
    echo "$version"
}

# Download and install binary
install_binary() {
    local platform="$1"
    local version="$2"
    local download_url="https://github.com/${REPO}/releases/download/${version}/lock-${platform}"
    local binary_path="${TEMP_DIR}/${BINARY_NAME}"
    
    print_info "Downloading lock-rs ${version} for ${platform}..."
    
    if ! curl -sL "$download_url" -o "$binary_path"; then
        print_error "Failed to download binary"
        exit 1
    fi
    
    if [ ! -f "$binary_path" ]; then
        print_error "Binary not found after download"
        exit 1
    fi
    
    chmod +x "$binary_path"
    
    print_info "Installing to ${INSTALL_DIR}..."
    
    if [ ! -d "$INSTALL_DIR" ]; then
        print_info "Creating directory ${INSTALL_DIR}..."
        sudo mkdir -p "$INSTALL_DIR"
    fi
    
    if sudo cp "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"; then
        print_success "lock-rs installed successfully!"
    else
        print_error "Failed to install binary"
        exit 1
    fi
}

# Verify installation
verify_installation() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local version
        version=$("$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        print_success "lock-rs is ready to use (${version})"
        print_info "Run 'lock --help' to get started"
    else
        print_warning "Installation completed but binary not found in PATH"
        print_info "Make sure ${INSTALL_DIR} is in your PATH"
    fi
}

# Main installation function
main() {
    print_info "Installing lock-rs..."
    
    # Check if curl is available
    if ! command -v curl >/dev/null 2>&1; then
        print_error "curl is required but not installed"
        exit 1
    fi
    
    # Check if tar is available
    if ! command -v tar >/dev/null 2>&1; then
        print_error "tar is required but not installed"
        exit 1
    fi
    
    local platform version
    
    platform=$(detect_platform)
    version=$(get_latest_version)
    
    print_info "Platform: ${platform}"
    print_info "Version: ${version}"
    
    # Check if already installed
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local current_version
        current_version=$("$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        print_warning "lock-rs is already installed (${current_version})"
        read -p "Do you want to update to ${version}? [y/N]: " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Installation cancelled"
            exit 0
        fi
    fi
    
    install_binary "$platform" "$version"
    verify_installation
}

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir DIR    Install binary to DIR (default: /usr/local/bin)"
            echo "  --help, -h          Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  INSTALL_DIR         Same as --install-dir"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run main function
main
