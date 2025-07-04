#!/bin/bash

# Rust MAME Launcher - Uninstaller
# Created by Edo Hikmahtiar - Indonesia

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

APP_NAME="rust-mame-launcher"

echo -e "${RED}================================================${NC}"
echo -e "${RED}  Rust MAME Launcher Uninstaller${NC}"
echo -e "${RED}================================================${NC}"
echo

# Determine installation locations
if [ -f "/usr/local/bin/$APP_NAME" ] || [ -f "/usr/bin/$APP_NAME" ]; then
    if [ "$EUID" -ne 0 ]; then 
        echo -e "${RED}System-wide installation detected.${NC}"
        echo -e "${RED}Please run with sudo: sudo ./uninstall.sh${NC}"
        exit 1
    fi
    SYSTEM_INSTALL=true
    if [ -f "/usr/local/bin/$APP_NAME" ]; then
        BIN_DIR="/usr/local/bin"
        SHARE_DIR="/usr/local/share"
    else
        BIN_DIR="/usr/bin"
        SHARE_DIR="/usr/share"
    fi
else
    SYSTEM_INSTALL=false
    BIN_DIR="$HOME/.local/bin"
    SHARE_DIR="$HOME/.local/share"
fi

echo -e "${YELLOW}This will remove Rust MAME Launcher from your system.${NC}"
read -p "Are you sure you want to continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstall cancelled."
    exit 0
fi

echo -e "\n${YELLOW}Removing files...${NC}"

# Remove binary
if [ -f "$BIN_DIR/$APP_NAME" ]; then
    rm -f "$BIN_DIR/$APP_NAME"
    echo -e "  ✓ Binary removed"
fi

# Remove desktop file
if [ -f "$SHARE_DIR/applications/$APP_NAME.desktop" ]; then
    rm -f "$SHARE_DIR/applications/$APP_NAME.desktop"
    echo -e "  ✓ Desktop file removed"
fi

# Remove icon
if [ -f "$SHARE_DIR/icons/hicolor/256x256/apps/$APP_NAME.png" ]; then
    rm -f "$SHARE_DIR/icons/hicolor/256x256/apps/$APP_NAME.png"
    echo -e "  ✓ Icon removed"
fi

# Remove documentation
if [ -d "$SHARE_DIR/doc/$APP_NAME" ]; then
    rm -rf "$SHARE_DIR/doc/$APP_NAME"
    echo -e "  ✓ Documentation removed"
fi

# Remove license
if [ -d "$SHARE_DIR/licenses/$APP_NAME" ]; then
    rm -rf "$SHARE_DIR/licenses/$APP_NAME"
    echo -e "  ✓ License removed"
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

echo -e "\n${GREEN}================================================${NC}"
echo -e "${GREEN}  Uninstall complete!${NC}"
echo -e "${GREEN}================================================${NC}"
echo
echo -e "${YELLOW}Note: User configuration files (if any) have been preserved.${NC}"
