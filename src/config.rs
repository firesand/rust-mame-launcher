use std::path::PathBuf;
use std::fs;
use crate::models::{AppConfig, VideoSettings};

pub fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("rust-mame-launcher");

    // Create directory if it doesn't exist
    let _ = fs::create_dir_all(&config_dir);

    config_dir.join("config.json")
}

pub fn save_config(config: &AppConfig, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

// Handle loading old config files with migration
pub fn load_config(path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;

    // Try to deserialize normally first
    match serde_json::from_str::<AppConfig>(&json) {
        Ok(config) => Ok(config),
        Err(_) => {
            // If that fails, try to load into a JSON value and migrate
            let mut value: serde_json::Value = serde_json::from_str(&json)?;

            // Add missing fields with default values
            if let serde_json::Value::Object(ref mut map) = value {
                // Add mame_audit_times if missing
                if !map.contains_key("mame_audit_times") {
                    map.insert("mame_audit_times".to_string(), serde_json::json!({}));

                    // If there's a last_audit_time, migrate it to the new format
                    if let Some(serde_json::Value::String(_last_audit)) = map.get("last_audit_time") {
                        if let Some(serde_json::Value::Number(selected_idx)) = map.get("selected_mame_index") {
                            if let Some(_idx) = selected_idx.as_u64() {
                                // We can't get the MAME identifier here, so we'll handle this in the app
                                println!("Note: Old audit time found, will migrate on next audit");
                            }
                        }
                    }
                }

                // NEW: Add icon-related fields if missing
                if !map.contains_key("icons_path") {
                    map.insert("icons_path".to_string(), serde_json::Value::Null);
                    println!("Migrated config: Added icons_path field");
                }

                if !map.contains_key("icon_size") {
                    map.insert("icon_size".to_string(), serde_json::json!(32));
                    println!("Migrated config: Added icon_size field (default: 32)");
                }

                if !map.contains_key("max_cached_icons") {
                    map.insert("max_cached_icons".to_string(), serde_json::json!(500));
                    println!("Migrated config: Added max_cached_icons field (default: 500)");
                }

                // If show_rom_icons is missing (from older versions)
                if !map.contains_key("show_rom_icons") {
                    map.insert("show_rom_icons".to_string(), serde_json::json!(true));
                    println!("Migrated config: Added show_rom_icons field (default: true)");
                }

                // ADD VIDEO SETTINGS MIGRATION
                if !map.contains_key("video_settings") {
                    map.insert("video_settings".to_string(),
                               serde_json::to_value(VideoSettings::default()).unwrap());
                    println!("Migrated config: Added video_settings field with defaults");
                }
            }

            // Now try to deserialize the modified JSON
            serde_json::from_value(value).map_err(|e| e.into())
        }
    }
}

pub fn get_mame_data_dir() -> PathBuf {
    let data_dir = dirs::home_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join(".mame")
    .join("rust-mame-launcher");

    // Create the directory structure if it doesn't exist
    let subdirs = [
        "nvram", "cfg", "sta", "diff", "inp", "snap", "ctrlr",
        "cheat", "crosshair", "artwork", "samples", "hash", "ui"
    ];

    for subdir in &subdirs {
        let _ = fs::create_dir_all(data_dir.join(subdir));
    }

    data_dir
}

// NEW: Helper function to validate and clean up icon path
pub fn validate_icon_path(path: &Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = path {
        if p.exists() && p.is_dir() {
            // Check if there are any .ico files in the directory
            if let Ok(entries) = fs::read_dir(p) {
                let has_icons = entries
                .filter_map(|e| e.ok())
                .any(|entry| {
                    entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("ico"))
                    .unwrap_or(false)
                });

                if has_icons {
                    return Some(p.clone());
                } else {
                    println!("Warning: Icon path exists but contains no .ico files: {:?}", p);
                }
            }
        } else {
            println!("Warning: Icon path does not exist or is not a directory: {:?}", p);
        }
    }
    None
}

// NEW: Get default icons path (check common locations)
pub fn get_default_icons_path(rom_dirs: &[PathBuf]) -> Option<PathBuf> {
    // Check common icon locations
    let possible_paths = vec![
        // Check in ROM directories for an "icons" subdirectory
        rom_dirs.iter()
        .map(|dir| dir.join("icons"))
        .collect::<Vec<_>>(),
        // Check in home directory
        vec![
            dirs::home_dir().map(|h| h.join(".mame").join("icons")),
            dirs::home_dir().map(|h| h.join("mame").join("icons")),
        ].into_iter().filter_map(|p| p).collect::<Vec<_>>(),
    ].into_iter().flatten();

    for path in possible_paths {
        if path.exists() && path.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                let has_icons = entries
                .filter_map(|e| e.ok())
                .any(|entry| {
                    entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("ico"))
                    .unwrap_or(false)
                });

                if has_icons {
                    println!("Found icons directory at: {:?}", path);
                    return Some(path);
                }
            }
        }
    }

    None
}
