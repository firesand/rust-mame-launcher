# Changelog

All notable changes to the Rust MAME Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-12-XX

### Added
- Multi-MAME version support allowing users to manage multiple MAME installations
- MAME Executables Manager dialog accessible via File menu
- Per-game MAME version preferences with memory
- Right-click context menu on games for MAME version selection
- Visual feedback showing active MAME version and total configured versions
- Preferred MAME version display in game details panel

### Changed
- Refactored launch system to support multiple MAME executables
- Updated main window to show active MAME information
- Modified ROM loading to work with selected MAME version
- Enhanced About dialog to show new version number

### Technical
- Added `MameExecutable` struct for managing MAME versions
- Added `game_preferred_mame` HashMap for storing preferences
- Converted single MAME executable to vector of executables
- Added context menu handling for game list

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Basic MAME frontend functionality
- ROM management from multiple directories
- Game artwork support (snapshots, cabinets, titles, artwork)
- Advanced filtering options
- Game metadata display
