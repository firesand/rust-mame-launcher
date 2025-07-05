#!/bin/bash

# Script to apply multi-MAME support patch
# Created by Edo Hikmahtiar - Indonesia

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}  Applying Multi-MAME Support Patch${NC}"
echo -e "${GREEN}================================================${NC}"
echo

# Check if we're in the project directory
if [ ! -f "Cargo.toml" ] || [ ! -f "src/main.rs" ]; then
    echo -e "${RED}Error: Not in project directory!${NC}"
    echo "Please run this script from your rust-mame-launcher directory"
    exit 1
fi

# Backup original files
echo -e "${YELLOW}Creating backups...${NC}"
cp Cargo.toml Cargo.toml.backup
cp src/main.rs src/main.rs.backup

# Apply the patch
echo -e "${YELLOW}Applying patch...${NC}"
patch -p1 < multi-mame-support.patch

# Update Cargo.toml dependencies if patch didn't work completely
if ! grep -q "serde =" Cargo.toml; then
    echo -e "${YELLOW}Adding dependencies to Cargo.toml...${NC}"
    cat >> Cargo.toml << 'EOF'
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "5.0"
EOF
fi

echo -e "${GREEN}Patch applied successfully!${NC}"
echo
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Build the project: cargo build --release"
echo "2. Install: sudo cp target/release/mame_gui /usr/local/bin/rust-mame-launcher"
echo "3. Run and configure multiple MAME versions!"
echo
echo -e "${GREEN}New features:${NC}"
echo "• File → MAME Executables → Add multiple MAME versions"
echo "• Per-game MAME version selection in the right panel"
echo "• Profile management for different MAME builds"
echo "• Configuration saved in ~/.config/rust-mame-launcher/"
