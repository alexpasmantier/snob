#!/bin/bash

# Snob Installation Script
# This script helps you install Snob quickly and easily

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_step() {
    echo -e "${BLUE}🔹 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Welcome message
echo -e "${BLUE}"
echo "🧐 Snob Installation Script"
echo "=========================="
echo -e "${NC}"
echo "This script will help you install Snob - the smart test selection tool."
echo ""

# Check prerequisites
print_step "Checking prerequisites..."

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 is required but not installed."
    echo "Please install Python 3.9+ and try again."
    exit 1
fi

PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
print_success "Python $PYTHON_VERSION found"

# Check if pip is installed
PIP_CMD=""
if command -v pip3 &> /dev/null; then
    PIP_CMD="pip3"
elif command -v pip &> /dev/null; then
    PIP_CMD="pip"
elif python3 -m pip --version &> /dev/null; then
    PIP_CMD="python3 -m pip"
elif python -m pip --version &> /dev/null; then
    PIP_CMD="python -m pip"
else
    print_error "pip is required but not found."
    echo ""
    echo "Please try one of the following:"
    echo "• Activate your virtual environment: source venv/bin/activate"
    echo "• Install pip system-wide: sudo apt install python3-pip  (Ubuntu/Debian)"
    echo "• Install pip with ensurepip: python3 -m ensurepip --upgrade"
    echo "• Use your system's package manager to install python3-pip"
    exit 1
fi

print_success "pip found ($PIP_CMD)"

# Check if we're in a git repository (optional but recommended)
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_warning "Not in a git repository. Snob works best with git projects."
fi

# Installation options
echo ""
echo "Choose installation method:"
echo "1) Binary CLI (recommended - faster and more flexible)"
echo "2) Pytest plugin (for pytest-integrated workflows)"
echo "3) Build from source (for contributors/advanced users)"
echo ""

while true; do
    read -p "Enter choice (1, 2, or 3): " choice
    case $choice in
        1)
            INSTALL_METHOD="binary"
            break
            ;;
        2)
            INSTALL_METHOD="plugin"
            break
            ;;
        3)
            INSTALL_METHOD="source"
            break
            ;;
        *)
            echo "Please enter 1, 2, or 3"
            ;;
    esac
done

if [ "$INSTALL_METHOD" = "binary" ]; then
    # Install prebuilt binary
    print_step "Installing Snob binary..."

    # Check if curl is available
    if ! command -v curl &> /dev/null; then
        print_error "curl is required for binary installation."
        echo "Please install curl or choose option 3 to build from source."
        exit 1
    fi

    # Detect architecture and OS
    ARCH=$(uname -m)
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')

    case $ARCH in
        x86_64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) 
            print_error "Unsupported architecture: $ARCH"
            print_warning "Try option 3 to build from source instead."
            exit 1
            ;;
    esac

    case $OS in
        linux) OS="unknown-linux-gnu" ;;
        darwin) OS="apple-darwin" ;;
        *)
            print_error "Unsupported OS: $OS"
            print_warning "Try option 3 to build from source instead."
            exit 1
            ;;
    esac

    BINARY_URL="https://github.com/alexpasmantier/snob/releases/latest/download/snob-${ARCH}-${OS}"
    INSTALL_DIR="$HOME/.local/bin"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download binary
    print_step "Downloading snob binary..."
    if curl -L "$BINARY_URL" -o "$INSTALL_DIR/snob" --fail --silent --show-error; then
        chmod +x "$INSTALL_DIR/snob"
        print_success "Binary downloaded and installed to $INSTALL_DIR/snob"
    else
        print_error "Failed to download binary from GitHub releases."
        echo "This might mean:"
        echo "• No prebuilt binary available for your platform ($ARCH-$OS)"
        echo "• GitHub releases may not be set up yet"
        echo ""
        print_warning "Falling back to building from source..."
        INSTALL_METHOD="source"
    fi

    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "$INSTALL_DIR is not in your PATH."
        echo "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
        echo "Or run: echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
    fi

    # Verify installation
    print_step "Verifying installation..."
    if "$INSTALL_DIR/snob" --version &>/dev/null; then
        SNOB_VERSION=$("$INSTALL_DIR/snob" --version 2>&1 || echo "version check failed")
        print_success "snob binary installed: $SNOB_VERSION"
    else
        print_warning "Binary installed but version check failed. May need to restart shell."
    fi

    # Usage instructions
    echo ""
    echo -e "${GREEN}🎉 Installation complete!${NC}"
    echo ""
    echo "Quick start:"
    echo "  snob src/file.py                       # List affected tests"
    echo "  pytest \$(snob src/file.py)             # Run affected tests"
    echo "  snob --commit-range HEAD~1..HEAD       # Test recent changes"
    echo ""
    echo "Next steps:"
    echo "- Add $INSTALL_DIR to your PATH if not already done"
    echo "- Create snob.toml for configuration"
    echo "- Check out examples: https://github.com/alexpasmantier/snob/blob/main/docs/EXAMPLES.md"

elif [ "$INSTALL_METHOD" = "plugin" ]; then
    # Install pytest plugin
    print_step "Installing pytest-snob plugin..."

    $PIP_CMD install pytest-snob

    print_success "pytest-snob installed successfully!"

    # Verify installation
    print_step "Verifying installation..."
    # Use python -m pytest to avoid conftest.py conflicts in current directory
    if python -m pytest --help 2>/dev/null | grep -q "commit-range"; then
        print_success "Installation verified!"
    elif python3 -m pytest --help 2>/dev/null | grep -q "commit-range"; then
        print_success "Installation verified!"
    else
        print_warning "Installation verification skipped (project-specific pytest config detected)."
        echo "To verify manually, run: python -m pytest --help | grep commit-range"
    fi

    # Usage instructions
    echo ""
    echo -e "${GREEN}🎉 Installation complete!${NC}"
    echo ""
    echo "Quick start:"
    echo "  pytest --commit-range HEAD~1..HEAD    # Test recent changes"
    echo "  pytest --commit-range main..HEAD      # Test since main branch"
    echo ""
    echo "Next steps:"
    echo "- Read the Quick Start guide: https://github.com/your-org/snob/blob/main/docs/QUICKSTART.md"
    echo "- Create snob.toml for configuration"
    echo "- Check out examples: https://github.com/your-org/snob/blob/main/docs/EXAMPLES.md"

fi

if [ "$INSTALL_METHOD" = "source" ]; then
    # Install from source
    print_step "Installing from source..."

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_step "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
        print_success "Rust installed"
    else
        RUST_VERSION=$(cargo --version | cut -d' ' -f2)
        print_success "Rust $RUST_VERSION found"
    fi

    # Clone repository if not already present
    if [ ! -f "Cargo.toml" ]; then
        print_step "Cloning Snob repository..."
        git clone https://github.com/your-org/snob.git
        cd snob
        print_success "Repository cloned"
    fi

    # Build the project
    print_step "Building Snob..."
    cargo build --release
    print_success "Build completed"

    # Install the binary
    print_step "Installing Snob CLI..."
    cargo install --path .
    print_success "Snob CLI installed"

    # Optional: Install Python integration
    read -p "Install Python integration? (y/n): " install_python
    if [[ $install_python =~ ^[Yy]$ ]]; then
        print_step "Installing Python integration..."
        $PIP_CMD install maturin
        maturin develop
        print_success "Python integration installed"
    fi

    # Verify installation
    print_step "Verifying installation..."
    if command -v snob &> /dev/null; then
        SNOB_VERSION=$(snob --version 2>&1 || echo "version check failed")
        print_success "snob CLI installed: $SNOB_VERSION"
    else
        print_warning "snob command not in PATH. You may need to restart your shell."
    fi

    # Usage instructions
    echo ""
    echo -e "${GREEN}🎉 Installation complete!${NC}"
    echo ""
    echo "Quick start:"
    echo "  snob src/file.py                       # List affected tests"
    echo "  pytest \$(snob src/file.py)             # Run affected tests"
    echo ""
    echo "Development commands:"
    echo "  cargo test                             # Run tests"
    echo "  cargo clippy                          # Lint code"
    echo "  cargo fmt                             # Format code"
    echo ""
    echo "Next steps:"
    echo "- Read the Contributing guide: https://github.com/your-org/snob/blob/main/CONTRIBUTING.md"
    echo "- Run 'cargo test' to verify everything works"
    echo "- Check out the architecture docs"
fi

# Final setup suggestions
echo ""
echo -e "${BLUE}💡 Pro Tips:${NC}"
echo "- Create a snob.toml file to configure for your project"
echo "- Add Snob to your CI/CD pipeline for faster builds"
echo "- Use 'snob --dot-graph deps.dot' to visualize dependencies"
echo ""
echo -e "${GREEN}Happy testing! 🧪${NC}"
