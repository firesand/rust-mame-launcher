use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use crate::graphics_presets::GraphicsConfig;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortColumn {
    Title,
    RomName,
    Year,
    Manufacturer,
    Status,
    PlayCount,
    LastPlayed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortColumn {
    fn default() -> Self {
        SortColumn::Title
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MameExecutable {
    pub name: String,
    pub path: String,
    pub version: String,
    pub total_games: usize,
    pub working_games: usize,
}

impl Default for MameExecutable {
    fn default() -> Self {
        Self {
            name: "Default MAME".to_string(),
            path: "mame".to_string(),
            version: String::new(),
            total_games: 0,
            working_games: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GameMetadata {
    pub name: String,
    pub description: String,
    pub year: String,
    pub manufacturer: String,
    pub controls: String,
    pub is_device: bool,
    pub is_bios: bool,
    pub is_mechanical: bool,
    pub runnable: bool,
    pub parent: Option<String>,
    pub is_clone: bool,
    pub driver_status: Option<String>,  // NEW
    pub emulation_status: Option<String>, // NEW
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RomStatus {
    Good,
    Imperfect,
    Preliminary,
    NotWorking,
}

impl RomStatus {
    pub fn to_icon(&self) -> &'static str {
        match self {
            RomStatus::Good => "✅",
            RomStatus::Imperfect => "⚠️",
            RomStatus::Preliminary => "🔧",
            RomStatus::NotWorking => "⛔",
        }
    }

    pub fn to_color(&self) -> egui::Color32 {
        match self {
            RomStatus::Good => egui::Color32::from_rgb(0, 255, 0),
            RomStatus::Imperfect => egui::Color32::from_rgb(255, 200, 0),
            RomStatus::Preliminary => egui::Color32::from_rgb(255, 150, 0),
            RomStatus::NotWorking => egui::Color32::from_rgb(255, 0, 0),
        }
    }
}

impl GameMetadata {
    pub fn get_status(&self) -> RomStatus {
        // Check driver status first
        if let Some(driver) = &self.driver_status {
            match driver.as_str() {
                "good" => return RomStatus::Good,
                "imperfect" => return RomStatus::Imperfect,
                "preliminary" => return RomStatus::Preliminary,
                _ => {}
            }
        }

        // Fall back to other checks
        if self.runnable {
            RomStatus::NotWorking
        } else if self.is_mechanical || self.emulation_status.as_deref() == Some("imperfect") {
            RomStatus::Imperfect
        } else {
            RomStatus::Good
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ArtTab {
    Snapshot,
    Cabinet,
    Title,
    Artwork,
    History,
}

impl Default for ArtTab {
    fn default() -> Self { ArtTab::Snapshot }
}

// NEW: Icon information for caching and management
#[derive(Debug, Clone)]
pub struct IconInfo {
    pub rom_name: String,
    pub loaded: bool,
    pub last_accessed: std::time::Instant,
}

// NEW: Icon load request for async loading
#[derive(Debug, Clone)]
pub struct IconLoadRequest {
    pub rom_name: String,
    pub priority: i32, // Higher priority loads first
    pub requested_at: std::time::Instant,
}

// NEW: Batch icon load result
#[derive(Debug)]
pub struct BatchIconLoadResult {
    pub rom_name: String,
    pub icon_data: Result<Vec<u8>, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterSettings {
    pub search_text: String,
    pub year_from: String,
    pub year_to: String,
    pub manufacturer: String,
    pub selected_manufacturers: Vec<String>,
    pub show_clones: bool,
    pub show_working_only: bool,  // Deprecated, kept for compatibility
    pub hide_non_games: bool,
    pub hide_mahjong: bool,
    pub hide_adult: bool,
    pub hide_casino: bool,
    pub show_favorites_only: bool,  // NEW
    pub status_filter: StatusFilter,  // NEW
}

// NEW: Status filter enum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StatusFilter {
    All,
    WorkingOnly,
    ImperfectOnly,
    NotWorkingOnly,
}

impl Default for StatusFilter {
    fn default() -> Self { StatusFilter::All }
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            year_from: String::new(),
            year_to: String::new(),
            manufacturer: String::new(),
            selected_manufacturers: Vec::new(),
            show_clones: false,
            show_working_only: false,
            hide_non_games: false,
            hide_mahjong: false,
            hide_adult: false,
            hide_casino: false,
            show_favorites_only: false,
            status_filter: StatusFilter::All,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RomSetType {
    Merged,
    Split,
    NonMerged,
    Unknown,
}

// NEW: Game statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameStats {
    pub play_count: u32,
    pub last_played: Option<String>,  // ISO timestamp
    pub total_play_time: u32,  // seconds
}

// NEW: Theme enum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    DarkBlue,
    DarkGrey,
    MidnightBlue,
    ArcadePurple,
    ClassicGreen,
    RetroOrange,
}

impl Theme {
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::DarkBlue,
            Theme::DarkGrey,
            Theme::MidnightBlue,
            Theme::ArcadePurple,
            Theme::ClassicGreen,
            Theme::RetroOrange,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theme::DarkBlue => "Dark Blue",
            Theme::DarkGrey => "Dark Grey",
            Theme::MidnightBlue => "Midnight Blue",
            Theme::ArcadePurple => "Arcade Purple",
            Theme::ClassicGreen => "Classic Green",
            Theme::RetroOrange => "Retro Orange",
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        match self {
            Theme::DarkBlue => {
                visuals.panel_fill = egui::Color32::from_rgb(20, 25, 40);
                visuals.window_fill = egui::Color32::from_rgb(25, 30, 45);
                visuals.extreme_bg_color = egui::Color32::from_rgb(15, 20, 35);
            }
            Theme::DarkGrey => {
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 35, 40);
                visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 25);
            }
            Theme::MidnightBlue => {
                visuals.panel_fill = egui::Color32::from_rgb(15, 20, 35);
                visuals.window_fill = egui::Color32::from_rgb(20, 30, 50);
                visuals.extreme_bg_color = egui::Color32::from_rgb(10, 15, 30);
            }
            Theme::ArcadePurple => {
                visuals.panel_fill = egui::Color32::from_rgb(25, 20, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 25, 45);
                visuals.extreme_bg_color = egui::Color32::from_rgb(15, 10, 25);
                visuals.selection.bg_fill = egui::Color32::from_rgb(100, 50, 150);
            }
            Theme::ClassicGreen => {
                visuals.panel_fill = egui::Color32::from_rgb(20, 30, 25);
                visuals.window_fill = egui::Color32::from_rgb(25, 35, 30);
                visuals.extreme_bg_color = egui::Color32::from_rgb(15, 25, 20);
                visuals.selection.bg_fill = egui::Color32::from_rgb(50, 150, 50);
            }
            Theme::RetroOrange => {
                visuals.panel_fill = egui::Color32::from_rgb(35, 25, 20);
                visuals.window_fill = egui::Color32::from_rgb(40, 30, 25);
                visuals.extreme_bg_color = egui::Color32::from_rgb(25, 15, 10);
                visuals.selection.bg_fill = egui::Color32::from_rgb(200, 100, 50);
            }
        }

        ctx.set_visuals(visuals);
    }
}

impl Default for Theme {
    fn default() -> Self { Theme::DarkBlue }
}

// ADD THIS NEW STRUCT FOR VIDEO SETTINGS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub video_backend: String,      // soft, opengl, d3d, bgfx
    pub window_mode: bool,          // Run in window
    pub maximize: bool,             // Start maximized
    pub wait_vsync: bool,           // Wait for vsync
    pub sync_refresh: bool,         // Sync to monitor refresh
    pub prescale: u8,               // Prescale factor (0-3)
    pub keep_aspect: bool,          // Keep aspect ratio
    pub filter: bool,               // Bilinear filtering
    pub num_screens: u8,            // Number of screens (1-4)
    pub custom_args: String,        // Additional custom arguments
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            video_backend: "auto".to_string(),
            window_mode: true,
            maximize: false,
            wait_vsync: false,
            sync_refresh: false,
            prescale: 0,
            keep_aspect: true,
            filter: true,
            num_screens: 1,
            custom_args: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub mame_executables: Vec<MameExecutable>,
    pub selected_mame_index: usize,
    pub rom_dirs: Vec<PathBuf>,
    pub extra_rom_dirs: Vec<PathBuf>,
    pub extra_asset_dirs: Vec<PathBuf>,
    pub filter_settings: FilterSettings,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub game_preferred_mame: HashMap<String, usize>,
    pub show_filters: bool,
    pub selected_rom: Option<String>,
    pub art_tab: ArtTab,
    pub use_mame_audit: bool,
    pub last_audit_time: Option<String>,
    pub mame_audit_times: HashMap<String, String>,
    pub assume_merged_sets: bool,
    pub favorite_games: HashSet<String>,  // NEW
    pub game_stats: HashMap<String, GameStats>,  // NEW
    pub theme: Theme,  // NEW
    pub show_rom_icons: bool,  // NEW

    // NEW: Additional icon-related fields
    pub icons_path: Option<PathBuf>,
    pub icon_size: u32,
    pub max_cached_icons: usize,

    pub graphics_config: GraphicsConfig,  // NEW
    pub video_settings: VideoSettings,  // ADD THIS LINE
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mame_executables: vec![],
            selected_mame_index: 0,
            rom_dirs: vec![],
            extra_rom_dirs: vec![],
            extra_asset_dirs: vec![],
            filter_settings: FilterSettings::default(),
            sort_column: SortColumn::default(),
            sort_direction: SortDirection::default(),
            game_preferred_mame: HashMap::new(),
            show_filters: false,
            selected_rom: None,
            art_tab: ArtTab::Snapshot,
            use_mame_audit: false,
            last_audit_time: None,
            mame_audit_times: HashMap::new(),
            assume_merged_sets: false,
            favorite_games: HashSet::new(),
            game_stats: HashMap::new(),
            theme: Theme::default(),
            show_rom_icons: true,

            // NEW: Icon defaults
            icons_path: None,
            icon_size: 32,
            max_cached_icons: 500,

            graphics_config: GraphicsConfig::default(),
            video_settings: VideoSettings::default(),  // ADD THIS LINE
        }
    }
}

// NEW: ROM collection statistics
#[derive(Debug, Default)]
pub struct RomStatistics {
    pub total_roms: usize,
    pub available_roms: usize,
    pub missing_roms: usize,
    pub total_clones: usize,
    pub total_parents: usize,
    pub roms_with_icons: usize,
    pub missing_icons: Vec<String>,
    pub working_count: usize,
    pub imperfect_count: usize,
    pub not_working_count: usize,
}
