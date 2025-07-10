use std::path::PathBuf;
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::thread;
use std::fs;
use std::process::Command;
use std::time::Instant;
use eframe::egui;
use crate::models::GameStats;

use crate::graphics_presets::GraphicsConfig;
use crate::config::{get_config_path, save_config, load_config, get_mame_data_dir};
use crate::models::{AppConfig, MameExecutable, GameMetadata, RomSetType, IconInfo};
use crate::mame_utils::{get_mame_version, load_mame_metadata_parallel_with_exec};
use crate::rom_utils::{
    collect_roms_with_zip_scan, load_roms_from_audit, detect_rom_set_type,
    load_rom_icon, get_parent_rom, ico_to_rgba_bytes
};

pub struct MyApp {
    pub config: AppConfig,
    pub graphics_config: GraphicsConfig,  // NEW
    pub mame_titles: HashMap<String, String>,
    pub roms: Vec<(String, String)>,
    pub roms_loading: bool,
    pub roms_tx: Option<mpsc::Receiver<Vec<(String, String)>>>,
    pub screenshot: Option<egui::ColorImage>,
    pub texture_handle: Option<egui::TextureHandle>,
    pub game_metadata: HashMap<String, GameMetadata>,
    pub art_texture: Option<egui::TextureHandle>,
    pub all_manufacturers: Vec<String>,
    pub show_about: bool,
    pub show_debug: bool,
    pub show_rom_diagnostics: bool,
    pub show_rom_set_info: bool,
    pub total_games_count: usize,
    pub working_games_count: usize,
    pub running_games: HashMap<String, (std::process::Child, std::time::Instant)>,
    pub mame_version: String,
    pub show_context_menu: bool,
    pub context_menu_position: egui::Pos2,
    pub context_menu_rom: Option<String>,
    pub show_mame_manager: bool,
    pub show_close_dialog: bool,
    pub pending_close: bool,
    pub config_path: PathBuf,
    pub expanded_parents: HashMap<String, bool>,
    pub manufacturer_search: String,
    pub manufacturer_dropdown_open: bool,
    pub audit_in_progress: bool,
    pub audit_progress: String,
    pub show_video_settings: bool,
    pub audit_tx: Option<mpsc::Receiver<String>>,
    pub rom_icons: HashMap<String, egui::TextureHandle>,
    pub default_icon_texture: Option<egui::TextureHandle>,
    pub audit_start_time: Option<std::time::Instant>, // NEW
    pub last_audit_progress: String, // NEW

    // NEW: Icon management fields
    pub icon_load_queue: VecDeque<String>,
    pub icon_info: HashMap<String, IconInfo>,
    pub last_icon_cleanup: Instant,
}

impl MyApp {
    pub fn new() -> Self {
        let config_path = get_config_path();
        let config = load_config(&config_path).unwrap_or_default();

        let mut app = Self {
            config,
            mame_titles: HashMap::new(),
            roms: vec![],
            roms_loading: false,
            roms_tx: None,
            screenshot: None,
            texture_handle: None,
            game_metadata: HashMap::new(),
            art_texture: None,
            all_manufacturers: Vec::new(),
            show_about: false,
            show_debug: false,
            show_rom_diagnostics: false,
            show_rom_set_info: false,
            total_games_count: 0,
            working_games_count: 0,
            running_games: HashMap::new(),
            mame_version: String::new(),
            show_context_menu: false,
            context_menu_position: egui::Pos2::ZERO,
            context_menu_rom: None,
            show_mame_manager: false,
            show_close_dialog: false,
            pending_close: false,
            config_path,
            expanded_parents: HashMap::new(),
            manufacturer_search: String::new(),
            manufacturer_dropdown_open: false,
            audit_in_progress: false,
            audit_progress: String::new(),
            audit_tx: None,
            show_video_settings: false,  // ADD THIS LINE
            rom_icons: HashMap::new(),
            default_icon_texture: None,
                graphics_config: GraphicsConfig::default(),
                audit_start_time: None, // NEW
                last_audit_progress: String::new(), // NEW

                // NEW: Initialize icon management fields
                icon_load_queue: VecDeque::new(),
                icon_info: HashMap::new(),
                last_icon_cleanup: Instant::now(),
        };

        // Load metadata if we have MAME configured
        if !app.config.mame_executables.is_empty() && app.config.selected_mame_index < app.config.mame_executables.len() {
            let mame_path = app.config.mame_executables[app.config.selected_mame_index].path.clone();
            app.load_mame_data(&mame_path);

            // Load ROMs if paths are configured
            if !app.config.rom_dirs.is_empty() {
                app.reload_roms();
            }
        }

        app
    }

    // NEW: Initialize default icon
    pub fn init_default_icon(&mut self, ctx: &egui::Context) {
        self.load_default_icon(ctx);
    }

    // NEW: Get icons path from extra_asset_dirs
    fn get_icons_path(&self) -> Option<PathBuf> {
        // Look for icons folder in extra_asset_dirs
        for asset_dir in &self.config.extra_asset_dirs {
            let icons_path = asset_dir.join("icons");
            if icons_path.exists() && icons_path.is_dir() {
                // Check if it contains .ico files
                if let Ok(entries) = std::fs::read_dir(&icons_path) {
                    let has_icons = entries
                    .filter_map(|e| e.ok())
                    .any(|entry| {
                        entry.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("ico"))
                        .unwrap_or(false)
                    });

                    if has_icons {
                        return Some(icons_path);
                    }
                }
            }
        }

        // Fallback to explicit icons_path if set
        self.config.icons_path.clone()
    }

    pub fn update_game_stats(&mut self, rom_name: &str, play_time: u32) {
        let stats = self.config.game_stats.entry(rom_name.to_string())
        .or_insert_with(GameStats::default);

        stats.play_count += 1;
        stats.last_played = Some(chrono::Local::now().to_rfc3339());
        stats.total_play_time += play_time;

        self.save_config();
    }

    pub fn check_running_games(&mut self) {
        let mut finished_games = Vec::new();
        let mut still_running = HashMap::new();

        // Take ownership of running_games temporarily
        let running_games = std::mem::take(&mut self.running_games);

        for (rom_name, (mut child, start_time)) in running_games {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Game has finished
                    let play_time = start_time.elapsed().as_secs() as u32;
                    finished_games.push((rom_name, play_time));
                }
                Ok(None) => {
                    // Still running, keep it
                    still_running.insert(rom_name, (child, start_time));
                }
                Err(e) => {
                    println!("Error checking game process: {}", e);
                }
            }
        }

        // Put back the still running games
        self.running_games = still_running;

        // Update stats for finished games
        for (rom_name, play_time) in finished_games {
            self.update_game_stats(&rom_name, play_time);
        }
    }

    // NEW: Load the default icon texture
    fn load_default_icon(&mut self, ctx: &egui::Context) {
        // Try to load a default.ico file first
        if let Some(icons_path) = self.get_icons_path() {
            let default_icon_path = icons_path.join("default.ico");
            if default_icon_path.exists() {
                if let Ok(img) = load_rom_icon("default", &icons_path) {
                    let rgba_bytes = ico_to_rgba_bytes(img, self.config.icon_size);
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [self.config.icon_size as usize, self.config.icon_size as usize],
                        &rgba_bytes,
                    );

                    self.default_icon_texture = Some(ctx.load_texture(
                        "default_icon",
                        color_image,
                        egui::TextureOptions::default(),
                    ));
                    return;
                }
            }
        }

        // Fallback: Create a simple default icon (gray square with "?")
        let size = self.config.icon_size as usize;
        let mut pixels = vec![80u8; size * size * 4]; // Dark gray

        // Simple pattern: make edges slightly darker
        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                if x == 0 || x == size - 1 || y == 0 || y == size - 1 {
                    pixels[idx] = 60;     // R
                    pixels[idx + 1] = 60; // G
                    pixels[idx + 2] = 60; // B
                    pixels[idx + 3] = 255; // A
                }
            }
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [size, size],
            &pixels,
        );

        self.default_icon_texture = Some(ctx.load_texture(
            "default_icon",
            color_image,
            egui::TextureOptions::default(),
        ));
    }

    // NEW: Queue an icon for loading
    pub fn queue_icon_load(&mut self, rom_name: String) {
        if !self.rom_icons.contains_key(&rom_name)
            && !self.icon_load_queue.contains(&rom_name)
            && !self.icon_info.contains_key(&rom_name) {
                self.icon_load_queue.push_back(rom_name);
            }
    }

    // NEW: Process queued icon loads (call this in update)
    pub fn process_icon_queue(&mut self, ctx: &egui::Context) {
        if !self.config.show_rom_icons {
            return;
        }

        let icons_path = match self.get_icons_path() {
            Some(path) => path,
            None => return,
        };

        // Process up to 5 icons per frame to avoid blocking
        for _ in 0..5 {
            if let Some(rom_name) = self.icon_load_queue.pop_front() {
                self.load_single_icon(&rom_name, &icons_path, ctx);
            } else {
                break;
            }
        }

        // Cleanup old icons periodically (every 30 seconds)
        if self.last_icon_cleanup.elapsed().as_secs() > 30 {
            self.cleanup_icon_cache();
            self.last_icon_cleanup = Instant::now();
        }
    }

    // NEW: Load a single icon
    fn load_single_icon(&mut self, rom_name: &str, icons_path: &PathBuf, ctx: &egui::Context) {
        let icon_size = self.config.icon_size;

        // Try to load the icon
        let icon_result = load_rom_icon(rom_name, icons_path);

        // If failed, try parent ROM for clones
        let icon_result = match icon_result {
            Ok(img) => Ok(img),
            Err(_) => {
                if let Some(parent) = get_parent_rom(rom_name, &self.game_metadata) {
                    load_rom_icon(&parent, icons_path)
                } else {
                    icon_result
                }
            }
        };

        // Convert and store the icon
        match icon_result {
            Ok(img) => {
                let rgba_bytes = ico_to_rgba_bytes(img, icon_size);
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [icon_size as usize, icon_size as usize],
                    &rgba_bytes,
                );

                let texture = ctx.load_texture(
                    format!("rom_icon_{}", rom_name),
                        color_image,
                        egui::TextureOptions::default(),
                );

                self.rom_icons.insert(rom_name.to_string(), texture);
                self.icon_info.insert(rom_name.to_string(), IconInfo {
                    rom_name: rom_name.to_string(),
                                      loaded: true,
                                      last_accessed: Instant::now(),
                });
            }
            Err(_) => {
                // Track failed loads to avoid retrying
                self.icon_info.insert(rom_name.to_string(), IconInfo {
                    rom_name: rom_name.to_string(),
                                      loaded: false,
                                      last_accessed: Instant::now(),
                });
            }
        }
    }

    // NEW: Get icon for a ROM (with fallback to default)
    pub fn get_rom_icon(&mut self, rom_name: &str) -> egui::TextureHandle {
        // Update last accessed time
        if let Some(info) = self.icon_info.get_mut(rom_name) {
            info.last_accessed = Instant::now();
        }

        // Return cached icon or default
        self.rom_icons
        .get(rom_name)
        .cloned()
        .or_else(|| self.default_icon_texture.clone())
        .unwrap_or_else(|| {
            // This shouldn't happen if default icon is properly loaded
            panic!("No default icon available!");
        })
    }

    // NEW: Cleanup old cached icons
    fn cleanup_icon_cache(&mut self) {
        if self.rom_icons.len() <= self.config.max_cached_icons {
            return;
        }

        // Find least recently used icons
        let mut icon_times: Vec<(String, Instant)> = self.icon_info
        .iter()
        .filter(|(name, info)| info.loaded && self.rom_icons.contains_key(*name))
        .map(|(name, info)| (name.clone(), info.last_accessed))
        .collect();

        icon_times.sort_by_key(|(_, time)| *time);

        // Remove oldest icons until we're under the limit
        let remove_count = self.rom_icons.len() - self.config.max_cached_icons;
        for (rom_name, _) in icon_times.iter().take(remove_count) {
            self.rom_icons.remove(rom_name);
            self.icon_info.remove(rom_name);
        }
    }

    // NEW: Preload icons for visible range
    pub fn preload_visible_icons(&mut self, visible_start: usize, visible_end: usize) {
        if !self.config.show_rom_icons || self.get_icons_path().is_none() {
            return;
        }

        const PRELOAD_RADIUS: usize = 10;
        let preload_start = visible_start.saturating_sub(PRELOAD_RADIUS);
        let preload_end = (visible_end + PRELOAD_RADIUS).min(self.roms.len());

        for idx in preload_start..preload_end {
            if let Some((_, rom_name)) = self.roms.get(idx) {
                if !self.rom_icons.contains_key(rom_name) && !self.icon_info.contains_key(rom_name) {
                    self.queue_icon_load(rom_name.clone());
                }
            }
        }
    }

    // NEW: Clear all icon cache
    pub fn clear_icon_cache(&mut self) {
        self.rom_icons.clear();
        self.icon_info.clear();
        self.icon_load_queue.clear();
    }

    // NEW: Reload default icon (e.g., after config change)
    pub fn reload_default_icon(&mut self, ctx: &egui::Context) {
        self.default_icon_texture = None;
        self.load_default_icon(ctx);
    }

    pub fn save_config(&self) {
        if let Err(e) = save_config(&self.config, &self.config_path) {
            eprintln!("Failed to save config: {}", e);
        }
    }

    pub fn toggle_favorite(&mut self, rom_name: &str) {
        if self.config.favorite_games.contains(rom_name) {
            self.config.favorite_games.remove(rom_name);
        } else {
            self.config.favorite_games.insert(rom_name.to_string());
        }
        self.save_config();
    }

    pub fn load_mame_data(&mut self, mame_path: &str) {
        self.mame_version = get_mame_version(mame_path);
        self.game_metadata = load_mame_metadata_parallel_with_exec(mame_path);
        self.mame_titles = self.game_metadata.iter().map(|(k, v)| (k.clone(), v.description.clone())).collect();

        // Count total and working games
        self.total_games_count = self.game_metadata.iter()
        .filter(|(_, meta)| !meta.is_device && !meta.is_bios)
        .count();
        self.working_games_count = self.game_metadata.iter()
        .filter(|(_, meta)| !meta.is_device && !meta.is_bios && !meta.is_mechanical && !meta.runnable)
        .count();

        // Extract unique manufacturers
        let mut manufacturers: Vec<String> = self.game_metadata.values()
        .map(|m| m.manufacturer.clone())
        .filter(|m| !m.is_empty())
        .collect();
        manufacturers.sort();
        manufacturers.dedup();
        self.all_manufacturers = manufacturers;
    }

    pub fn reload_roms(&mut self) {
        // Clear icon cache when reloading ROMs
        self.clear_icon_cache();

        if !self.config.mame_executables.is_empty() && self.config.selected_mame_index < self.config.mame_executables.len() {
            let rom_dirs = self.config.rom_dirs.clone();
            let mame_titles = self.mame_titles.clone();
            let mame_executable = self.config.mame_executables[self.config.selected_mame_index].path.clone();
            let use_audit = self.config.use_mame_audit;
            let metadata = self.game_metadata.clone();
            let audit_file_path = self.get_audit_file_path(self.config.selected_mame_index);
            let assume_merged = self.config.assume_merged_sets;  // NEW: Get the config value

            let (tx, rx) = mpsc::channel();
            self.roms_tx = Some(rx);
            self.roms_loading = true;
            self.audit_progress = "Starting ROM scan...".to_string(); // Reuse audit progress for ROM loading

            // Add a progress sender
            let (progress_tx, progress_rx) = mpsc::channel();

            thread::spawn(move || {
                let _ = progress_tx.send("Scanning directories...".to_string()); // NEW
                let roms = if use_audit {
                    if let Some(audit_path) = audit_file_path {
                        // Try to load from audit data
                        println!("Loading ROMs from audit data for current MAME version...");
                        let audit_roms = load_roms_from_audit(&mame_executable, &mame_titles, &metadata, &audit_path);

                        if audit_roms.is_empty() {
                            println!("No ROMs found in audit file, falling back to enhanced scanning...");
                            let _ = progress_tx.send("Processing ROM files (enhanced scan)...".to_string()); // NEW
                            // Use enhanced scanner that looks inside ZIPs
                            collect_roms_with_zip_scan(&rom_dirs, &mame_titles, &metadata, assume_merged)
                        } else {
                            audit_roms
                        }
                    } else {
                        println!("Could not determine audit file path, using enhanced scanning...");
                        let _ = progress_tx.send("Processing ROM files (enhanced scan)...".to_string()); // NEW
                        // Use enhanced scanner that looks inside ZIPs
                        collect_roms_with_zip_scan(&rom_dirs, &mame_titles, &metadata, assume_merged)
                    }
                } else {
                    // Use enhanced scanner that looks inside ZIPs for merged sets
                    println!("Loading ROMs via enhanced ZIP scanning...");
                    let _ = progress_tx.send("Processing ROM files (enhanced scan)...".to_string()); // NEW
                    collect_roms_with_zip_scan(&rom_dirs, &mame_titles, &metadata, assume_merged)
                };

                println!("Total ROMs found: {}", roms.len());
                let _ = tx.send(roms);
            });

            // Store progress receiver
            self.audit_tx = Some(progress_rx);

        } else {
            println!("Cannot reload ROMs: No MAME executable configured");
        }
    }

    /// Get the path to the audit file for a specific MAME executable
    pub fn get_audit_file_path(&self, mame_index: usize) -> Option<PathBuf> {
        if let Some(mame) = self.config.mame_executables.get(mame_index) {
            // Create a unique filename based on MAME version or path hash
            let mame_id = self.get_mame_identifier(mame);
            let filename = format!("mame_avail_{}.ini", mame_id);

            let audit_dir = get_mame_data_dir().join("ui");
            let _ = fs::create_dir_all(&audit_dir);

            Some(audit_dir.join(filename))
        } else {
            None
        }
    }

    /// Generate a unique identifier for a MAME executable
    pub fn get_mame_identifier(&self, mame: &MameExecutable) -> String {
        // Use a combination of name and version, sanitized for filesystem
        let combined = format!("{}_{}", mame.name, mame.version);
        combined.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
    }

    /// Check if an audit file exists for the current MAME
    pub fn has_audit_file(&self) -> bool {
        if let Some(audit_path) = self.get_audit_file_path(self.config.selected_mame_index) {
            audit_path.exists()
        } else {
            false
        }
    }

    pub fn run_mame_audit(&mut self) {
        if self.config.mame_executables.is_empty() || self.audit_in_progress {
            return;
        }

        let mame_index = self.config.selected_mame_index;
        let mame_path = self.config.mame_executables[mame_index].path.clone();
        let rom_dirs = self.config.rom_dirs.clone();
        let extra_dirs = self.config.extra_rom_dirs.clone();

        // Get our custom audit file location
        let audit_file_path = self.get_audit_file_path(mame_index);
        if audit_file_path.is_none() {
            println!("Error: Could not determine audit file path");
            return;
        }
        let audit_file_path = audit_file_path.unwrap();

        self.audit_in_progress = true;
        self.audit_progress = "Starting ROM audit...".to_string();
        self.audit_start_time = Some(std::time::Instant::now()); // NEW

        let (tx, rx) = mpsc::channel();
        self.audit_tx = Some(rx);

        thread::spawn(move || {
            let _ = tx.send("Preparing MAME audit...".to_string());

            // Build rompath argument
            let all_dirs = rom_dirs.iter().chain(extra_dirs.iter());
            // MAME uses semicolon as separator on all platforms
            let separator = ";";
            let rom_paths = all_dirs
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(separator);

            // MAME needs a ui directory to write mame_avail.ini
            let mame_dir = std::path::Path::new(&mame_path).parent().unwrap_or(std::path::Path::new("."));
            let mame_ui_dir = mame_dir.join("ui");
            let _ = fs::create_dir_all(&mame_ui_dir);

            // Remove any existing mame_avail.ini to ensure fresh audit
            let mame_avail_path = mame_ui_dir.join("mame_avail.ini");
            let _ = fs::remove_file(&mame_avail_path);

            let _ = tx.send("Running MAME audit (scanning inside all ROM archives)...".to_string());

            // Run MAME audit - this scans inside every ZIP and creates mame_avail.ini
            let output = Command::new(&mame_path)
            .current_dir(&mame_dir)  // Run in MAME's directory
            .arg("-rompath")
            .arg(&rom_paths)
            .arg("-verifyroms")
            .output();

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Parse audit results from output
                    let mut total_roms = 0;
                    let mut good_roms = 0;
                    let mut bad_roms = 0;
                    let mut not_found = 0;

                    for line in stdout.lines() {
                        if line.contains("romset") {
                            total_roms += 1;
                            if line.contains("is good") || line.contains("is best available") {
                                good_roms += 1;
                            } else if line.contains("is bad") {
                                bad_roms += 1;
                            } else if line.contains("NOT FOUND") {
                                not_found += 1;
                            }
                        }
                    }

                    let _ = tx.send(format!(
                        "Audit complete: {} ROMs scanned, {} good, {} bad, {} not found",
                        total_roms, good_roms, bad_roms, not_found
                    ));

                    // Give MAME a moment to finish writing the file
                    thread::sleep(std::time::Duration::from_millis(500));

                    // Check if MAME created mame_avail.ini and copy it to our location
                    if mame_avail_path.exists() {
                        match fs::copy(&mame_avail_path, &audit_file_path) {
                            Ok(_) => {
                                let _ = tx.send(format!("Audit file saved for this MAME version"));

                                // Clean up MAME's ui directory
                                let _ = fs::remove_file(&mame_avail_path);
                            }
                            Err(e) => {
                                let _ = tx.send(format!("Failed to copy audit file: {}", e));
                            }
                        }
                    } else {
                        // Sometimes MAME writes to a different location, check common alternatives
                        let alt_locations = vec![
                            PathBuf::from(".mame").join("ui").join("mame_avail.ini"),
                      dirs::home_dir().unwrap_or_default().join(".mame").join("ui").join("mame_avail.ini"),
                      PathBuf::from("ui").join("mame_avail.ini"),
                        ];

                        let mut found = false;
                        for alt_path in alt_locations {
                            if alt_path.exists() {
                                match fs::copy(&alt_path, &audit_file_path) {
                                    Ok(_) => {
                                        let _ = tx.send(format!("Audit file found at {:?} and saved", alt_path));
                                        let _ = fs::remove_file(&alt_path);
                                        found = true;
                                        break;
                                    }
                                    Err(e) => {
                                        println!("Failed to copy from {:?}: {}", alt_path, e);
                                    }
                                }
                            }
                        }

                        if !found {
                            let _ = tx.send("Warning: Could not find mame_avail.ini after audit".to_string());
                            let _ = tx.send("MAME may have written it to a different location".to_string());
                        }
                    }

                    let _ = tx.send("AUDIT_COMPLETE".to_string());
                }
                Err(e) => {
                    let _ = tx.send(format!("Failed to run MAME audit: {}", e));
                    let _ = tx.send("AUDIT_FAILED".to_string());
                }
            }
        });
    }

    /// Clean up audit file when removing a MAME executable
    pub fn cleanup_audit_file(&mut self, mame_index: usize) {
        if let Some(audit_path) = self.get_audit_file_path(mame_index) {
            if audit_path.exists() {
                match fs::remove_file(&audit_path) {
                    Ok(_) => println!("Removed audit file: {:?}", audit_path),
                    Err(e) => println!("Failed to remove audit file: {}", e),
                }
            }
        }

        // Also remove from the audit times map
        if let Some(mame) = self.config.mame_executables.get(mame_index) {
            let mame_id = self.get_mame_identifier(mame);
            self.config.mame_audit_times.remove(&mame_id);
        }
    }

    /// List all audit files in the ui directory
    pub fn list_audit_files(&self) -> Vec<(String, PathBuf)> {
        let mut audit_files = Vec::new();
        let audit_dir = get_mame_data_dir().join("ui");

        if let Ok(entries) = fs::read_dir(&audit_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("mame_avail_") && filename.ends_with(".ini") {
                        audit_files.push((filename.to_string(), path));
                    }
                }
            }
        }

        audit_files
    }

    /// Clean up orphaned audit files (for MAME executables that no longer exist)
    pub fn cleanup_orphaned_audit_files(&mut self) {
        let audit_files = self.list_audit_files();
        let mut valid_audit_files = std::collections::HashSet::new();

        // Collect valid audit file names
        for (idx, _) in self.config.mame_executables.iter().enumerate() {
            if let Some(audit_path) = self.get_audit_file_path(idx) {
                if let Some(filename) = audit_path.file_name().and_then(|n| n.to_str()) {
                    valid_audit_files.insert(filename.to_string());
                }
            }
        }

        // Remove orphaned files
        for (filename, path) in audit_files {
            if !valid_audit_files.contains(&filename) {
                match fs::remove_file(&path) {
                    Ok(_) => println!("Removed orphaned audit file: {}", filename),
                    Err(e) => println!("Failed to remove orphaned audit file {}: {}", filename, e),
                }
            }
        }
    }

    pub fn debug_rom_loading(&self) -> String {
        let mut debug_info = String::new();

        debug_info.push_str(&format!("MAME Executables: {}\n", self.config.mame_executables.len()));
        if !self.config.mame_executables.is_empty() {
            let current = &self.config.mame_executables[self.config.selected_mame_index];
            debug_info.push_str(&format!("Current MAME: {} ({})\n", current.name, current.path));
        }

        debug_info.push_str(&format!("\nROM Directories: {}\n", self.config.rom_dirs.len()));
        for (i, dir) in self.config.rom_dirs.iter().enumerate() {
            debug_info.push_str(&format!("  {}: {:?} (exists: {})\n",
                                         i + 1, dir, dir.exists()));
        }

        debug_info.push_str(&format!("\nExtra Asset Directories: {}\n", self.config.extra_asset_dirs.len()));
        for (i, dir) in self.config.extra_asset_dirs.iter().enumerate() {
            debug_info.push_str(&format!("  {}: {:?} (exists: {})\n",
                                         i + 1, dir, dir.exists()));
        }

        debug_info.push_str(&format!("\nROMs Status:\n"));
        debug_info.push_str(&format!("  Total ROMs loaded: {}\n", self.roms.len()));
        debug_info.push_str(&format!("  Loading in progress: {}\n", self.roms_loading));
        debug_info.push_str(&format!("  Use MAME audit: {}\n", self.config.use_mame_audit));
        debug_info.push_str(&format!("  Assume merged sets: {}\n", self.config.assume_merged_sets));

        // NEW: Add icon status
        debug_info.push_str(&format!("\nIcon Status:\n"));
        debug_info.push_str(&format!("  Show ROM icons: {}\n", self.config.show_rom_icons));

        // Check for icons folder in extra_asset_dirs
        let icons_path = self.get_icons_path();
        if let Some(path) = icons_path {
            debug_info.push_str(&format!("  Icons path (auto-detected): {:?}\n", path));
        } else {
            debug_info.push_str(&format!("  Icons path: Not found in Extra Asset directories\n"));
        }

        debug_info.push_str(&format!("  Icons cached: {}\n", self.rom_icons.len()));
        debug_info.push_str(&format!("  Icons in queue: {}\n", self.icon_load_queue.len()));
        debug_info.push_str(&format!("  Icon size: {}px\n", self.config.icon_size));
        debug_info.push_str(&format!("  Max cached icons: {}\n", self.config.max_cached_icons));

        if let Some(current_mame) = self.config.mame_executables.get(self.config.selected_mame_index) {
            let mame_id = self.get_mame_identifier(current_mame);
            if let Some(last_audit) = self.config.mame_audit_times.get(&mame_id) {
                debug_info.push_str(&format!("  Last audit time: {}\n", last_audit));
            }
        }

        debug_info.push_str(&format!("\nGame Metadata: {} entries\n", self.game_metadata.len()));
        debug_info.push_str(&format!("MAME Titles: {} entries\n", self.mame_titles.len()));

        debug_info.push_str(&format!("\nFilter Settings:\n"));
        debug_info.push_str(&format!("  Hide non-games: {}\n", self.config.filter_settings.hide_non_games));
        debug_info.push_str(&format!("  Show clones: {}\n", self.config.filter_settings.show_clones));
        debug_info.push_str(&format!("  Show working only: {}\n", self.config.filter_settings.show_working_only));
        debug_info.push_str(&format!("  Hide adult: {}\n", self.config.filter_settings.hide_adult));
        debug_info.push_str(&format!("  Hide casino: {}\n", self.config.filter_settings.hide_casino));
        debug_info.push_str(&format!("  Hide mahjong: {}\n", self.config.filter_settings.hide_mahjong));

        debug_info
    }

    /// Diagnostic function to help users understand their ROM situation
    pub fn diagnose_rom_setup(&self) -> String {
        let mut report = String::new();

        report.push_str("=== ROM Setup Diagnostics ===\n\n");

        // Check ROM directories
        report.push_str("ROM Directories:\n");
        for dir in &self.config.rom_dirs {
            if dir.exists() {
                let zip_count = std::fs::read_dir(dir)
                .map(|entries| {
                    entries.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("zip"))
                        .unwrap_or(false)
                    })
                    .count()
                })
                .unwrap_or(0);

                report.push_str(&format!("  ✓ {:?} - {} ZIP files found\n", dir, zip_count));
            } else {
                report.push_str(&format!("  ✗ {:?} - DIRECTORY NOT FOUND\n", dir));
            }
        }

        // NEW: Check icon directory
        report.push_str("\nIcon Directory:\n");
        let icons_path = self.get_icons_path();
        if let Some(path) = icons_path {
            if path.exists() {
                let ico_count = std::fs::read_dir(&path)
                .map(|entries| {
                    entries.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("ico"))
                        .unwrap_or(false)
                    })
                    .count()
                })
                .unwrap_or(0);

                report.push_str(&format!("  ✓ {:?} - {} ICO files found\n", path, ico_count));
            } else {
                report.push_str(&format!("  ✗ {:?} - DIRECTORY NOT FOUND\n", path));
            }
        } else {
            report.push_str("  ✗ No 'icons' folder found in Extra Asset directories\n");
            if !self.config.extra_asset_dirs.is_empty() {
                report.push_str("    Checked in:\n");
                for dir in &self.config.extra_asset_dirs {
                    report.push_str(&format!("      - {}\n", dir.display()));
                }
            }
        }

        report.push_str("\nROM Set Type Detection:\n");

        // Try to detect ROM set type by sampling
        let mut has_clones_as_files = false;
        let mut sample_games = Vec::new();

        for dir in &self.config.rom_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten().take(20) {  // Sample first 20 files
                    if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        sample_games.push(name.to_string());

                        // Check if this looks like a clone (has a letter suffix)
                        if let Some(meta) = self.game_metadata.get(name) {
                            if meta.is_clone {
                                has_clones_as_files = true;
                            }
                        }
                    }
                }
            }
        }

        if has_clones_as_files {
            report.push_str("  Detected: SPLIT or NON-MERGED ROM set\n");
            report.push_str("  (Clone ROMs exist as separate files)\n");
            report.push_str("  ✓ Directory scanning will work fine\n");
        } else if !sample_games.is_empty() {
            report.push_str("  Detected: Likely MERGED ROM set\n");
            report.push_str("  (No separate clone files found)\n");
            report.push_str("  ⚠ Audit required to see clones inside parent ZIPs\n");
        }

        report.push_str("\nAudit Status:\n");
        if self.config.use_mame_audit {
            report.push_str("  ✓ Audit mode ENABLED\n");

            if let Some(audit_path) = self.get_audit_file_path(self.config.selected_mame_index) {
                if audit_path.exists() {
                    // Count entries in audit file
                    if let Ok(contents) = std::fs::read_to_string(&audit_path) {
                        let available_count = contents.lines()
                        .skip_while(|line| !line.contains("[AVAILABLE]"))
                        .skip(1)
                        .take_while(|line| !line.starts_with('['))
                        .filter(|line| line.contains(" = "))
                        .count();

                        report.push_str(&format!("  ✓ Audit file exists: {} games listed\n", available_count));

                        if let Some(current_mame) = self.config.mame_executables.get(self.config.selected_mame_index) {
                            let mame_id = self.get_mame_identifier(current_mame);
                            if let Some(audit_time) = self.config.mame_audit_times.get(&mame_id) {
                                report.push_str(&format!("  Last audit: {}\n", audit_time));
                            }
                        }
                    }
                } else {
                    report.push_str("  ✗ No audit file found!\n");
                    report.push_str("  → Run 'ROM Audit' from Options menu\n");
                }
            }
        } else {
            report.push_str("  ✗ Audit mode DISABLED\n");
            report.push_str("  → Enable 'Use audit data' for merged ROM support\n");
        }

        report.push_str("\nCurrent ROM Loading:\n");
        report.push_str(&format!("  Games loaded: {}\n", self.roms.len()));

        if self.roms.is_empty() && !self.config.rom_dirs.is_empty() {
            report.push_str("\n⚠ TROUBLESHOOTING: No games loaded!\n");
            report.push_str("Possible causes:\n");
            report.push_str("  1. Using merged ROMs without running audit\n");
            report.push_str("  2. ROM files not in ZIP format\n");
            report.push_str("  3. Incorrect ROM directory path\n");
            report.push_str("  4. All ROMs filtered out by current filter settings\n");
        }

        report
    }

    pub fn get_rom_set_type(&self) -> RomSetType {
        detect_rom_set_type(&self.config.rom_dirs, &self.game_metadata)
    }

    pub fn get_missing_parent_roms(&self) -> Vec<(String, String)> {
        let mut missing = Vec::new();

        for (_rom_display, rom_name) in &self.roms {
            if let Some(metadata) = self.game_metadata.get(rom_name) {
                if let Some(parent) = &metadata.parent {
                    // Check if parent ROM exists in our collection
                    let parent_exists = self.roms.iter()
                    .any(|(_, name)| name == parent);

                    if !parent_exists {
                        missing.push((rom_name.clone(), parent.clone()));
                    }
                }
            }
        }

        missing
    }

    pub fn debug_parent_clone_relationships(&self) {
        println!("\n=== DEBUG: Parent/Clone Analysis ===");

        // Count parent and clone games in metadata
        let mut parent_count = 0;
        let mut clone_count = 0;
        let mut parent_to_clones: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for (name, meta) in &self.game_metadata {
            if meta.is_clone {
                clone_count += 1;
                if let Some(parent) = &meta.parent {
                    parent_to_clones.entry(parent.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());
                }
            } else if !meta.is_device && !meta.is_bios {
                parent_count += 1;
            }
        }

        println!("Total metadata entries: {}", self.game_metadata.len());
        println!("Parent games: {}", parent_count);
        println!("Clone games: {}", clone_count);
        println!("Parents with clones: {}", parent_to_clones.len());

        // Show some examples
        println!("\nFirst 10 parents with clones:");
        for (parent, clones) in parent_to_clones.iter().take(10) {
            println!("  {} -> {} clones: {:?}", parent, clones.len(),
                     clones.iter().take(3).collect::<Vec<_>>());
        }

        // Check specific well-known parent/clone relationships
        println!("\nChecking known parent/clone games:");
        let test_games = vec!["sf2", "pacman", "mslug", "kof98", "dino", "captcomm", "1943", "simpsons"];

        for game in test_games {
            if let Some(meta) = self.game_metadata.get(game) {
                println!("\n{}: is_clone={}, parent={:?}", game, meta.is_clone, meta.parent);

                // Find its clones
                if let Some(clones) = parent_to_clones.get(game) {
                    println!("  Has {} clones: {:?}", clones.len(), clones.iter().take(5).collect::<Vec<_>>());
                }
            } else {
                println!("\n{}: NOT FOUND in metadata", game);
            }
        }

        // Check what ROMs we actually have loaded
        println!("\n=== Loaded ROMs Analysis ===");
        println!("Total ROMs loaded: {}", self.roms.len());

        let mut loaded_parents = 0;
        let mut loaded_clones = 0;

        for (_, rom_name) in &self.roms {
            if let Some(meta) = self.game_metadata.get(rom_name) {
                if meta.is_clone {
                    loaded_clones += 1;
                } else if !meta.is_device && !meta.is_bios {
                    loaded_parents += 1;
                }
            }
        }

        println!("Loaded parent ROMs: {}", loaded_parents);
        println!("Loaded clone ROMs: {}", loaded_clones);

        // Sample some loaded ROMs
        println!("\nFirst 20 loaded ROMs:");
        for (display, name) in self.roms.iter().take(20) {
            if let Some(meta) = self.game_metadata.get(name) {
                println!("  {} [{}] - clone: {}, parent: {:?}",
                         display, name, meta.is_clone, meta.parent);
            }
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            graphics_config: GraphicsConfig::default(),
            mame_titles: HashMap::new(),
            roms: Vec::new(),
            roms_loading: false,
            roms_tx: None,
            screenshot: None,
            texture_handle: None,
            game_metadata: HashMap::new(),
            art_texture: None,
            all_manufacturers: Vec::new(),
            show_about: false,
            show_debug: false,
            show_rom_diagnostics: false,
            show_rom_set_info: false,
            total_games_count: 0,
            working_games_count: 0,
            running_games: HashMap::new(),
            mame_version: String::new(),
            show_context_menu: false,
            context_menu_position: egui::Pos2::ZERO,
            context_menu_rom: None,
            show_mame_manager: false,
            show_close_dialog: false,
            pending_close: false,
            config_path: get_config_path(),
            expanded_parents: HashMap::new(),
            manufacturer_search: String::new(),
            manufacturer_dropdown_open: false,
            audit_in_progress: false,
            audit_progress: String::new(),
            audit_tx: None,
            show_video_settings: false,
            rom_icons: HashMap::new(),
            default_icon_texture: None,
                audit_start_time: None,
                last_audit_progress: String::new(),

                // NEW: Initialize icon management fields
                icon_load_queue: VecDeque::new(),
                icon_info: HashMap::new(),
                last_icon_cleanup: Instant::now(),
        }
    }
}
