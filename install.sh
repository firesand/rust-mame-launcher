#!/bin/bash

# Rust MAME Launcher - Universal Linux Installer
# Created by Edo Hikmahtiar - Indonesia

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Application info
APP_NAME="rust-mame-launcher"
APP_VERSION="0.1.0"
INSTALL_PREFIX="${PREFIX:-/usr/local}"

echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}  Rust MAME Launcher Installer v${APP_VERSION}${NC}"
echo -e "${GREEN}  Created by Edo Hikmahtiar - Indonesia${NC}"
echo -e "${GREEN}================================================${NC}"
echo

# Check if running as root for system-wide install
if [ "$EUID" -eq 0 ] || [ -n "$SUDO_USER" ]; then 
    echo -e "${YELLOW}Installing system-wide to ${INSTALL_PREFIX}${NC}"
    SYSTEM_INSTALL=true
    BIN_DIR="${INSTALL_PREFIX}/bin"
    SHARE_DIR="${INSTALL_PREFIX}/share"
else
    echo -e "${YELLOW}Installing for current user only${NC}"
    SYSTEM_INSTALL=false
    BIN_DIR="$HOME/.local/bin"
    SHARE_DIR="$HOME/.local/share"
    INSTALL_PREFIX="$HOME/.local"
fi

# Check for required dependencies
echo -e "\n${YELLOW}Checking dependencies...${NC}"

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "  ✓ $1 found"
        return 0
    else
        echo -e "  ${RED}✗ $1 not found${NC}"
        return 1
    fi
}

# Check build dependencies
BUILD_DEPS_MISSING=false
if ! check_command "cargo"; then
    BUILD_DEPS_MISSING=true
fi

if ! check_command "rustc"; then
    BUILD_DEPS_MISSING=true
fi

# Check runtime dependencies
RUNTIME_DEPS_MISSING=false
if ! check_command "mame"; then
    echo -e "  ${YELLOW}! MAME not found (optional but recommended)${NC}"
fi

if [ "$BUILD_DEPS_MISSING" = true ]; then
    echo -e "\n${RED}Build dependencies missing!${NC}"
    echo "Please install Rust toolchain:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Create directories
echo -e "\n${YELLOW}Creating directories...${NC}"
mkdir -p "$BIN_DIR"
mkdir -p "$SHARE_DIR/applications"
mkdir -p "$SHARE_DIR/icons/hicolor/256x256/apps"
mkdir -p "$SHARE_DIR/doc/$APP_NAME"

# Build the application
echo -e "\n${YELLOW}Building Rust MAME Launcher...${NC}"
if [ -f "Cargo.toml" ]; then
    cargo build --release
    
    if [ ! -f "target/release/mame_gui" ]; then
        echo -e "${RED}Build failed! Binary not found.${NC}"
        exit 1
    fi
else
    echo -e "${RED}Cargo.toml not found! Please run this script from the project root.${NC}"
    exit 1
fi

# Install binary
echo -e "\n${YELLOW}Installing binary...${NC}"
cp -f "target/release/mame_gui" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"
echo -e "  ✓ Binary installed to $BIN_DIR/$APP_NAME"

# Install desktop file
echo -e "\n${YELLOW}Installing desktop file...${NC}"
cat > "$SHARE_DIR/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=Rust MAME Launcher
Comment=A MAME frontend built with Rust
Exec=$BIN_DIR/$APP_NAME
Icon=$APP_NAME
Terminal=false
Type=Application
Categories=Game;Emulator;
Keywords=mame;arcade;emulator;games;
StartupNotify=true
EOF
chmod 644 "$SHARE_DIR/applications/$APP_NAME.desktop"
echo -e "  ✓ Desktop file installed"

# Install icon
echo -e "\n${YELLOW}Installing icon...${NC}"
if [ -f "assets/mame-frontend-icon.png" ]; then
    cp -f "assets/mame-frontend-icon.png" "$SHARE_DIR/icons/hicolor/256x256/apps/$APP_NAME.png"
    echo -e "  ✓ Icon installed"
else
    echo -e "  ${YELLOW}! Icon not found, skipping${NC}"
fi

# Install documentation
echo -e "\n${YELLOW}Installing documentation...${NC}"
if [ -f "README.md" ]; then
    cp -f "README.md" "$SHARE_DIR/doc/$APP_NAME/"
    echo -e "  ✓ README installed"
fi

if [ -f "LICENSE" ]; then
    mkdir -p "$SHARE_DIR/licenses/$APP_NAME"
    cp -f "LICENSE" "$SHARE_DIR/licenses/$APP_NAME/"
    echo -e "  ✓ License installed"
fi

# Update desktop database and icon cache (if system install)
if [ "$SYSTEM_INSTALL" = true ]; then
    echo -e "\n${YELLOW}Updating system databases...${NC}"
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$SHARE_DIR/applications" 2>/dev/null || true
    fi
    
    if command -v gtk-update-icon-cache &> /dev/null; then
        gtk-update-icon-cache -f -t "$SHARE_DIR/icons/hicolor" 2>/dev/null || true
    fi
fi

# Add to PATH if needed
if [ "$SYSTEM_INSTALL" = false ]; then
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        echo -e "\n${YELLOW}Adding $BIN_DIR to PATH...${NC}"
        echo "" >> "$HOME/.bashrc"
        echo "# Added by Rust MAME Launcher installer" >> "$HOME/.bashrc"
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$HOME/.bashrc"
        echo -e "  ${YELLOW}! Please restart your terminal or run: source ~/.bashrc${NC}"
    fi
fi

echo -e "\n${GREEN}================================================${NC}"
echo -e "${GREEN}  Installation complete!${NC}"
echo -e "${GREEN}================================================${NC}"
echo
echo -e "You can now run Rust MAME Launcher:"
echo -e "  • From terminal: ${GREEN}$APP_NAME${NC}"
echo -e "  • From application menu: ${GREEN}Rust MAME Launcher${NC}"
echo
echo -e "To uninstall, run: ${YELLOW}./uninstall.sh${NC}"
echo
