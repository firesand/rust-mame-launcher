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
