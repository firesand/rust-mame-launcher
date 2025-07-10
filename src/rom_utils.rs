use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use image;
//use image::io::Reader as ImageReader; depracated
use image::ImageReader;
use image::DynamicImage;
use eframe::egui;
use zip::ZipArchive;
use crate::models::{GameMetadata, FilterSettings, RomSetType, StatusFilter, RomStatus};

/// Apply filters to ROMs based on filter settings
pub fn apply_rom_filters(
    filters: &FilterSettings,
    metadata: &HashMap<String, GameMetadata>,
    display_name: &str,
    rom_name: &str,
    favorites: &HashSet<String>,
) -> bool {
    // Favorites filter
    if filters.show_favorites_only && !favorites.contains(rom_name) {
        return false;
    }

    // Search text filter
    if !filters.search_text.is_empty() {
        let search_lower = filters.search_text.to_lowercase();
        let display_lower = display_name.to_lowercase();
        let rom_lower = rom_name.to_lowercase();

        if !display_lower.contains(&search_lower) && !rom_lower.contains(&search_lower) {
            return false;
        }
    }

    // Get metadata for this ROM
    if let Some(meta) = metadata.get(rom_name) {
        // Filter non-games
        if filters.hide_non_games && (meta.is_device || meta.is_bios) {
            return false;
        }

        // Status filter
        match filters.status_filter {
            StatusFilter::All => {},
            StatusFilter::WorkingOnly => {
                if meta.get_status() != RomStatus::Good {
                    return false;
                }
            },
            StatusFilter::ImperfectOnly => {
                if meta.get_status() != RomStatus::Imperfect {
                    return false;
                }
            },
            StatusFilter::NotWorkingOnly => {
                if meta.get_status() != RomStatus::NotWorking {
                    return false;
                }
            },
        }

        // Year filter
        if !filters.year_from.is_empty() || !filters.year_to.is_empty() {
            if let Ok(year) = meta.year.parse::<u32>() {
                if !filters.year_from.is_empty() {
                    if let Ok(from_year) = filters.year_from.parse::<u32>() {
                        if year < from_year {
                            return false;
                        }
                    }
                }
                if !filters.year_to.is_empty() {
                    if let Ok(to_year) = filters.year_to.parse::<u32>() {
                        if year > to_year {
                            return false;
                        }
                    }
                }
            }
        }

        // Manufacturer filter
        if !filters.manufacturer.is_empty() && meta.manufacturer != filters.manufacturer {
            return false;
        }

        // Content filters
        let lower_desc = meta.description.to_lowercase();

        if filters.hide_mahjong && (lower_desc.contains("mahjong") || lower_desc.contains("mah-jong")) {
            return false;
        }

        if filters.hide_adult && (lower_desc.contains("adult") || lower_desc.contains("nude")) {
            return false;
        }

        if filters.hide_casino && (lower_desc.contains("casino") || lower_desc.contains("poker") ||
            lower_desc.contains("slot") || lower_desc.contains("cards")) {
            return false;
            }
    }

    true
}

/// Collect ROMs from directories by scanning for ZIP files
pub fn collect_roms_from_dirs(
    rom_dirs: &[PathBuf],
    _mame_executable: &str,
    mame_titles: &HashMap<String, String>
) -> Vec<(String, String)> {
    let mut roms = Vec::new();

    for dir in rom_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let display_name = mame_titles.get(stem)
                        .cloned()
                        .unwrap_or_else(|| stem.to_string());
                        roms.push((display_name, stem.to_string()));
                    }
                }
            }
        }
    }

    // Sort by display name
    roms.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    roms
}

/// Enhanced ROM collection that scans inside ZIPs for merged sets
pub fn collect_roms_with_zip_scan(
    rom_dirs: &[PathBuf],
    mame_titles: &HashMap<String, String>,
    metadata: &HashMap<String, GameMetadata>,
    assume_merged: bool,  // NEW PARAMETER
) -> Vec<(String, String)> {
    let mut roms = Vec::new();
    let mut found_roms = HashSet::new();

    // First, collect all parent-clone relationships
    let mut parent_to_clones: HashMap<String, Vec<String>> = HashMap::new();
    let mut clone_to_parent: HashMap<String, String> = HashMap::new();

    for (name, meta) in metadata {
        if let Some(parent) = &meta.parent {
            parent_to_clones.entry(parent.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());
            clone_to_parent.insert(name.clone(), parent.clone());
        }
    }

    println!("Scanning ROM directories with ZIP inspection...");
    if assume_merged {
        println!("  Assuming merged ROM sets (all clones in parent ZIPs)");
    }

    for dir in rom_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        // Add the ZIP file itself (parent ROM)
                        if !found_roms.contains(stem) {
                            let display_name = mame_titles.get(stem)
                            .cloned()
                            .unwrap_or_else(|| stem.to_string());
                            roms.push((display_name, stem.to_string()));
                            found_roms.insert(stem.to_string());
                        }

                        // For merged sets: check if this parent ZIP contains clones
                        if let Some(clones) = parent_to_clones.get(stem) {
                            if assume_merged {
                                // If assume_merged is true, add all clones without checking ZIP contents
                                for clone in clones {
                                    if !found_roms.contains(clone) {
                                        let display_name = mame_titles.get(clone)
                                        .cloned()
                                        .unwrap_or_else(|| clone.to_string());
                                        roms.push((display_name, clone.to_string()));
                                        found_roms.insert(clone.to_string());
                                        println!("  Assumed clone {} in {}.zip (merged set mode)", clone, stem);
                                    }
                                }
                            } else {
                                // Otherwise, check ZIP contents
                                if let Ok(file) = File::open(&path) {
                                    if let Ok(mut archive) = ZipArchive::new(file) {
                                        // Get list of files in the ZIP
                                        let mut files_in_zip = Vec::new();
                                        for i in 0..archive.len() {
                                            if let Ok(file) = archive.by_index(i) {
                                                let full_name = file.name().to_string();
                                                if let Some(filename) = full_name.split('/').last() {
                                                    files_in_zip.push(filename.to_string());
                                                }
                                            }
                                        }

                                        // Check each clone
                                        for clone in clones {
                                            if should_show_clone(clone, &files_in_zip, metadata) {
                                                if !found_roms.contains(clone) {
                                                    let display_name = mame_titles.get(clone)
                                                    .cloned()
                                                    .unwrap_or_else(|| clone.to_string());
                                                    roms.push((display_name, clone.to_string()));
                                                    found_roms.insert(clone.to_string());
                                                    println!("  Found clone {} inside {}.zip", clone, stem);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by display name
    roms.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    println!("Total ROMs found (including clones in merged sets): {}", roms.len());
    roms
}

/// Check if a clone's ROM files exist in the ZIP
fn should_show_clone(clone_name: &str, files_in_zip: &[String], metadata: &HashMap<String, GameMetadata>) -> bool {
    // For merged ROM sets, clone ROMs can be in the parent ZIP with various naming patterns:
    // 1. Named after the clone (e.g., "1944j.ic42" for clone 1944j)
    // 2. Shared with parent (many ROMs are common between parent and clones)
    // 3. Sometimes with different extensions or numbers

    // First, check if any files are explicitly named after the clone
    let clone_prefix = format!("{}.", clone_name);
    let has_clone_files = files_in_zip.iter().any(|f| f.starts_with(&clone_prefix));

    if has_clone_files {
        return true;
    }

    // For merged sets, if this is a known clone (has metadata),
    // we can assume it's included in the parent ZIP
    // This is generally true for properly merged ROM sets
    if metadata.contains_key(clone_name) {
        // Additional check: make sure the ZIP has a reasonable number of files
        // (merged sets typically have many files)
        if files_in_zip.len() > 5 {
            return true;
        }
    }

    false
}

/// Load ROMs from MAME audit file
pub fn load_roms_from_audit(
    _mame_executable: &str,
    mame_titles: &HashMap<String, String>,
    metadata: &HashMap<String, GameMetadata>,
    audit_file_path: &Path
) -> Vec<(String, String)> {
    let mut roms = Vec::new();

    if let Ok(contents) = fs::read_to_string(audit_file_path) {
        // Find the [AVAILABLE] section
        let mut in_available_section = false;

        for line in contents.lines() {
            if line.trim() == "[AVAILABLE]" {
                in_available_section = true;
                continue;
            }

            if in_available_section && line.starts_with('[') {
                // We've reached another section, stop
                break;
            }

            if in_available_section && line.contains(" = 1") {
                // Extract ROM name (before the " = 1")
                if let Some(rom_name) = line.split(" = ").next() {
                    let rom_name = rom_name.trim();

                    // Skip BIOS and devices
                    if let Some(meta) = metadata.get(rom_name) {
                        if meta.is_bios || meta.is_device {
                            continue;
                        }
                    }

                    let display_name = mame_titles.get(rom_name)
                    .cloned()
                    .unwrap_or_else(|| rom_name.to_string());

                    roms.push((display_name, rom_name.to_string()));
                }
            }
        }
    }

    // Sort by display name
    roms.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    roms
}

/// Load artwork image for a ROM
pub fn load_art_image(
    rom_name: &str,
    extra_asset_dirs: &[PathBuf],
    art_type: &str
) -> Option<egui::ColorImage> {
    // Map art type to folder names
    let folder = match art_type {
        "snapshot" => "snap",
        "cabinet" => "cabinets",
        "title" => "titles",
        "artwork" => "artwork",
        "flyer" => "flyers",
        "marquee" => "marquees",
        _ => "snap", // default to snapshots
    };

    // Look for artwork in various formats
    let extensions = ["png", "jpg", "jpeg", "bmp"];

    for dir in extra_asset_dirs {
        // First try in the specific subfolder
        let subfolder_path = dir.join(folder);
        if subfolder_path.exists() {
            for ext in &extensions {
                let path = subfolder_path.join(format!("{}.{}", rom_name, ext));
                if path.exists() {
                    if let Ok(data) = fs::read(&path) {
                        // Convert image data to ColorImage
                        if let Ok(image) = image::load_from_memory(&data) {
                            let size = [image.width() as _, image.height() as _];
                            let image_buffer = image.to_rgba8();
                            let pixels = image_buffer.as_flat_samples();
                            return Some(egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                pixels.as_slice(),
                            ));
                        }
                    }
                }
            }
        }

        // Fallback: check in root of asset directory
        for ext in &extensions {
            let path = dir.join(format!("{}.{}", rom_name, ext));
            if path.exists() {
                if let Ok(data) = fs::read(&path) {
                    // Convert image data to ColorImage
                    if let Ok(image) = image::load_from_memory(&data) {
                        let size = [image.width() as _, image.height() as _];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        return Some(egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_slice(),
                        ));
                    }
                }
            }
        }
    }

    None
}

// ============= NEW ICON-RELATED FUNCTIONS =============

/// Load a ROM icon from the icons directory
pub fn load_rom_icon(rom_name: &str, icons_path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let icon_path = icons_path.join(format!("{}.ico", rom_name));

    if !icon_path.exists() {
        return Err(format!("Icon not found for ROM: {}", rom_name).into());
    }

    let reader = ImageReader::open(&icon_path)?;
    let img = reader.decode()?;

    Ok(img)
}

/// Get parent ROM name for clones
pub fn get_parent_rom(rom_name: &str, game_metadata: &HashMap<String, GameMetadata>) -> Option<String> {
    game_metadata.get(rom_name)
    .and_then(|metadata| metadata.parent.clone())
}

/// Convert ico to RGBA bytes for egui
pub fn ico_to_rgba_bytes(image: DynamicImage, target_size: u32) -> Vec<u8> {
    let rgba_image = image
    .resize_exact(target_size, target_size, image::imageops::FilterType::Lanczos3)
    .to_rgba8();

    rgba_image.into_raw()
}

/// Batch load icons for a range of ROMs
pub fn preload_icons_range(
    roms: &[String],
    icons_path: &Path,
    start_idx: usize,
    end_idx: usize,
    icon_size: u32,
) -> Vec<(String, Result<Vec<u8>, String>)> {
    let mut results = Vec::new();

    for idx in start_idx..end_idx.min(roms.len()) {
        let rom_name = &roms[idx];
        let result = match load_rom_icon(rom_name, icons_path) {
            Ok(img) => Ok(ico_to_rgba_bytes(img, icon_size)),
            Err(e) => Err(e.to_string()),
        };
        results.push((rom_name.clone(), result));
    }

    results
}

/// Check if an icon exists for a ROM
pub fn icon_exists(rom_name: &str, icons_path: &Path) -> bool {
    let icon_path = icons_path.join(format!("{}.ico", rom_name));
    icon_path.exists()
}

/// Load icon with fallback to parent ROM icon
pub fn load_rom_icon_with_fallback(
    rom_name: &str,
    icons_path: &Path,
    game_metadata: &HashMap<String, GameMetadata>
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Try to load the icon for this ROM
    match load_rom_icon(rom_name, icons_path) {
        Ok(img) => Ok(img),
        Err(_) => {
            // If failed, try parent ROM for clones
            if let Some(parent) = get_parent_rom(rom_name, game_metadata) {
                load_rom_icon(&parent, icons_path)
            } else {
                Err(format!("No icon found for ROM: {} (and no parent ROM)", rom_name).into())
            }
        }
    }
}

/// Get statistics about ROM icons
pub fn get_icon_statistics(
    rom_names: &[String],
    icons_path: &Path,
    game_metadata: &HashMap<String, GameMetadata>
) -> (usize, Vec<String>) {
    let mut icons_found = 0;
    let mut missing_icons = Vec::new();

    for rom_name in rom_names {
        if icon_exists(rom_name, icons_path) {
            icons_found += 1;
        } else {
            // Check if parent has icon
            let parent_has_icon = get_parent_rom(rom_name, game_metadata)
            .map(|parent| icon_exists(&parent, icons_path))
            .unwrap_or(false);

            if !parent_has_icon {
                missing_icons.push(rom_name.clone());
            } else {
                icons_found += 1; // Count as found since parent has icon
            }
        }
    }

    (icons_found, missing_icons)
}

/// Create a default icon (simple colored square with "?")
pub fn create_default_icon(size: u32) -> Vec<u8> {
    let pixels = vec![128u8; (size * size * 4) as usize]; // Gray background

    // This is a simplified placeholder - in production, you'd want to:
    // 1. Load an actual default.ico file, or
    // 2. Generate a more sophisticated default icon with text rendering

    pixels
}

// ============= END NEW ICON-RELATED FUNCTIONS =============

/// Detect ROM set type by analyzing the ROM files
pub fn detect_rom_set_type(
    rom_dirs: &[PathBuf],
    metadata: &HashMap<String, GameMetadata>
) -> RomSetType {
    let mut has_separate_clones = false;
    let mut total_files = 0;
    let mut clone_files = 0;

    // Sample ROM files to detect the set type
    for dir in rom_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    total_files += 1;

                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Some(meta) = metadata.get(stem) {
                            if meta.is_clone {
                                clone_files += 1;
                                has_separate_clones = true;
                            }
                        }
                    }

                    // Sample enough files to make a determination
                    if total_files > 50 {
                        break;
                    }
                }
            }
        }
    }

    if total_files == 0 {
        RomSetType::Unknown
    } else if has_separate_clones {
        // If we found clone files, it's either split or non-merged
        if clone_files as f32 / total_files as f32 > 0.3 {
            RomSetType::Split
        } else {
            RomSetType::NonMerged
        }
    } else {
        // No separate clone files found, likely merged
        RomSetType::Merged
    }
}

/// Alternative: Detect ROM set type by looking inside ZIPs
pub fn detect_rom_set_type_enhanced(rom_dirs: &[PathBuf], metadata: &HashMap<String, GameMetadata>) -> RomSetType {
    let mut has_separate_clone_zips = false;
    let mut has_clones_in_parent_zips = false;
    let mut sample_count = 0;

    // Build parent-clone map
    let mut parent_to_clones: HashMap<String, Vec<String>> = HashMap::new();
    for (name, meta) in metadata {
        if let Some(parent) = &meta.parent {
            parent_to_clones.entry(parent.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());
        }
    }

    'outer: for dir in rom_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        sample_count += 1;

                        // Check if this is a clone ZIP
                        if metadata.get(stem).map(|m| m.is_clone).unwrap_or(false) {
                            has_separate_clone_zips = true;
                        }

                        // Check if this parent ZIP contains clones
                        if parent_to_clones.contains_key(stem) {
                            if let Ok(file) = File::open(&path) {
                                if let Ok(mut archive) = ZipArchive::new(file) {
                                    // Look for clone files inside
                                    for i in 0..archive.len() {
                                        if let Ok(file) = archive.by_index(i) {
                                            let file_name = file.name().to_string();
                                            // Check if this file belongs to a clone
                                            for clone in parent_to_clones[stem].iter() {
                                                if file_name.contains(clone) {
                                                    has_clones_in_parent_zips = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Sample enough files
                        if sample_count > 20 {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    if has_clones_in_parent_zips && !has_separate_clone_zips {
        RomSetType::Merged
    } else if has_separate_clone_zips {
        RomSetType::Split
    } else if sample_count > 0 {
        RomSetType::NonMerged
    } else {
        RomSetType::Unknown
    }
}
