#!/bin/bash

# Build release packages for Rust MAME Launcher
# Created by Edo Hikmahtiar - Indonesia

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

VERSION="0.1.0"
APP_NAME="rust-mame-launcher"

echo -e "${GREEN}Building release packages for Rust MAME Launcher v${VERSION}${NC}"

# Create release directory
mkdir -p "release"

# Build the application
echo -e "\n${YELLOW}Building release binary...${NC}"
cargo build --release

# Create tarball for general Linux
echo -e "\n${YELLOW}Creating Linux tarball...${NC}"
mkdir -p "release/${APP_NAME}-${VERSION}"
cp -r "target/release/mame_gui" "release/${APP_NAME}-${VERSION}/"
cp -r "Cargo.toml" "release/${APP_NAME}-${VERSION}/"
cp -r "Cargo.lock" "release/${APP_NAME}-${VERSION}/"
cp -r "src" "release/${APP_NAME}-${VERSION}/"
cp "install.sh" "release/${APP_NAME}-${VERSION}/"
cp "uninstall.sh" "release/${APP_NAME}-${VERSION}/"
cp "rust-mame-launcher.desktop" "release/${APP_NAME}-${VERSION}/" 2>/dev/null || true

# Copy assets if they exist
if [ -d "assets" ]; then
    cp -r "assets" "release/${APP_NAME}-${VERSION}/"
fi

# Create README if it doesn't exist
if [ ! -f "README.md" ]; then
    cat > "release/${APP_NAME}-${VERSION}/README.md" << EOF
# Rust MAME Launcher

A MAME frontend built with Rust and egui

Created by Edo Hikmahtiar - Indonesia

## Features

- Multi-folder ROM management
- Game artwork display (snapshots, cabinets, titles, artwork)
- Advanced search and filtering
- MAME metadata integration
- Content filtering (hide adult/casino/mahjong games)
- Support for ROMs, CHDs, and BIOS files

## Installation

### General Linux:
\`\`\`bash
chmod +x install.sh
./install.sh
\`\`\`

### Arch Linux:
\`\`\`bash
makepkg -si
\`\`\`

## Usage

Run from terminal:
\`\`\`bash
rust-mame-launcher
\`\`\`

Or launch from your application menu.

## Requirements

- MAME (optional but recommended)
- GTK3 libraries
- X11 or Wayland display server

## License

MIT License
EOF
else
    cp "README.md" "release/${APP_NAME}-${VERSION}/"
fi

# Create LICENSE if it doesn't exist
if [ ! -f "LICENSE" ]; then
    cat > "release/${APP_NAME}-${VERSION}/LICENSE" << EOF
MIT License

Copyright (c) 2024 Edo Hikmahtiar

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
else
    cp "LICENSE" "release/${APP_NAME}-${VERSION}/"
fi

# Make scripts executable
chmod +x "release/${APP_NAME}-${VERSION}/install.sh"
chmod +x "release/${APP_NAME}-${VERSION}/uninstall.sh"

# Create tarball
cd release
tar -czf "${APP_NAME}-${VERSION}-linux-x86_64.tar.gz" "${APP_NAME}-${VERSION}"
cd ..

# Create AppImage (optional, requires linuxdeploy)
if command -v linuxdeploy &> /dev/null; then
    echo -e "\n${YELLOW}Creating AppImage...${NC}"
    mkdir -p "release/AppDir/usr/bin"
    mkdir -p "release/AppDir/usr/share/applications"
    mkdir -p "release/AppDir/usr/share/icons/hicolor/256x256/apps"
    
    cp "target/release/mame_gui" "release/AppDir/usr/bin/${APP_NAME}"
    cp "rust-mame-launcher.desktop" "release/AppDir/usr/share/applications/" 2>/dev/null || true
    
    if [ -f "assets/mame-frontend-icon.png" ]; then
        cp "assets/mame-frontend-icon.png" "release/AppDir/usr/share/icons/hicolor/256x256/apps/${APP_NAME}.png"
    fi
    
    cd release
    linuxdeploy --appdir AppDir --output appimage || echo "AppImage creation failed"
    cd ..
fi

# Create Arch package
echo -e "\n${YELLOW}Creating Arch Linux package...${NC}"
mkdir -p "release/arch-pkg"
cp "PKGBUILD" "release/arch-pkg/" 2>/dev/null || true
cp -r "release/${APP_NAME}-${VERSION}" "release/arch-pkg/"
cd "release/arch-pkg"
# Note: makepkg would be run by the user, not here
cd ../..

echo -e "\n${GREEN}================================================${NC}"
echo -e "${GREEN}  Release packages created in ./release/${NC}"
echo -e "${GREEN}================================================${NC}"
echo
echo "Available packages:"
echo -e "  • Linux tarball: ${YELLOW}release/${APP_NAME}-${VERSION}-linux-x86_64.tar.gz${NC}"
if [ -f "release/${APP_NAME}-x86_64.AppImage" ]; then
    echo -e "  • AppImage: ${YELLOW}release/${APP_NAME}-x86_64.AppImage${NC}"
fi
echo -e "  • Arch package files: ${YELLOW}release/arch-pkg/${NC}"
echo
echo "To distribute:"
echo "  1. Upload the tarball to GitHub releases"
echo "  2. Users can download and run ./install.sh"
echo "  3. Arch users can use the PKGBUILD"
