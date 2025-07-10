use std::process::Command;
use std::collections::HashMap;
use std::path::PathBuf;
use rayon::prelude::*;
use crate::models::{GameMetadata, VideoSettings};
use crate::config::get_mame_data_dir;
use crate::graphics_presets::GraphicsConfig;

pub fn get_mame_version(exec_path: &str) -> String {
    if let Ok(output) = Command::new(exec_path)
        .arg("-version")
        .output() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            version_str.lines().next().unwrap_or("Unknown").to_string()
        } else {
            "Unknown".to_string()
        }
}

pub fn load_mame_metadata_parallel_with_exec(exec_path: &str) -> HashMap<String, GameMetadata> {
    println!("Running MAME -listxml...");
    let output = Command::new(exec_path)
    .arg("-listxml")
    .output()
    .expect("Failed to run mame -listxml");

    let xml_str = String::from_utf8_lossy(&output.stdout);
    println!("XML output size: {} bytes", xml_str.len());

    let entries: Vec<_> = xml_str.split("<machine ").skip(1).collect();
    println!("Found {} machine entries", entries.len());

    let metadata: HashMap<String, GameMetadata> = entries
    .into_par_iter()
    .filter_map(|entry| {
        // The machine tag attributes are on the first line
        let first_line = entry.lines().next()?;

        // Extract name attribute
        let name = first_line.split("name=\"")
        .nth(1)?
        .split('"')
        .next()?;

        // Check for parent/clone status - look for both cloneof and romof
        let parent = if first_line.contains("cloneof=\"") {
            first_line.split("cloneof=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .map(|s| s.to_string())
        } else if first_line.contains("romof=\"") {
            // Some MAME versions use romof instead of/in addition to cloneof
            first_line.split("romof=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .map(|s| s.to_string())
        } else {
            None
        };
        let is_clone = parent.is_some();

        // Debug clone detection for specific games
        if name.starts_with("1944") || name.starts_with("simpsons") || name.starts_with("1943") || name == "kov2" {
            println!("METADATA DEBUG - ROM {}: parent={:?}, is_clone={}", name, parent, is_clone);
            if first_line.len() > 100 {
                println!("  First line: {}...", &first_line[..100]);
            } else {
                println!("  First line: {}", first_line);
            }
        }

        // Check if it's a device, BIOS, or mechanical machine
        let is_device = entry.contains("isdevice=\"yes\"");
        let is_bios = entry.contains("isbios=\"yes\"");
        let is_mechanical = entry.contains("ismechanical=\"yes\"");
        let runnable = entry.contains("runnable=\"no\"");

        let description = entry.lines()
        .find(|l| l.contains("<description>"))
        .and_then(|l| l.split_once('>'))
        .and_then(|(_, r)| r.split_once('<'))
        .map(|(d, _)| d.trim().to_string())
        .unwrap_or_default();

        let year = entry.lines()
        .find(|l| l.contains("<year>"))
        .and_then(|l| l.split_once('>'))
        .and_then(|(_, r)| r.split_once('<'))
        .map(|(d, _)| d.trim().to_string())
        .unwrap_or_default();

        let manufacturer = entry.lines()
        .find(|l| l.contains("<manufacturer>"))
        .and_then(|l| l.split_once('>'))
        .and_then(|(_, r)| r.split_once('<'))
        .map(|(d, _)| d.trim().to_string())
        .unwrap_or_default();

        let controls = entry.lines()
        .find(|l| l.contains("<control "))
        .and_then(|l| l.split("type=\"").nth(1))
        .and_then(|s| s.split('"').next())
        .unwrap_or_default()
        .to_string();

        Some((
            name.to_string(),
              GameMetadata {
                  name: name.to_string(),            // ← FIXED
              description,                       // ← FIXED
              year,
              manufacturer,
              controls,                          // ← FIXED
              is_device,
              is_bios,
              is_mechanical,
              runnable,
              parent,
              is_clone,
              driver_status: None,
              emulation_status: None,
              }
        ))
    })
    .collect();

    // Debug summary
    let clone_count = metadata.values().filter(|m| m.is_clone).count();
    let parent_count = metadata.values().filter(|m| !m.is_clone && !m.is_device && !m.is_bios).count();

    println!("\nMetadata parsing complete:");
    println!("  Total games: {}", metadata.len());
    println!("  Parent games: {}", parent_count);
    println!("  Clone games: {}", clone_count);

    // Show a few examples of parsed clones
    println!("\nExample parsed clones:");
    for (name, meta) in metadata.iter().filter(|(_, m)| m.is_clone).take(5) {
        println!("  {} -> parent: {:?}", name, meta.parent);
    }

    metadata
}

// UPDATED FUNCTION WITH VIDEO SETTINGS
pub fn launch_rom_with_mame_tracked(
    rom_name: &str,
    rom_dirs: &[PathBuf],
    extra_rom_dirs: &[PathBuf],
    mame_executable: &str,
    graphics_config: &GraphicsConfig,
    video_settings: &VideoSettings,  // ADD THIS PARAMETER
) -> Result<std::process::Child, Box<dyn std::error::Error>> {
    let mut cmd = Command::new(mame_executable);

    // Build rompath argument
    let all_dirs = rom_dirs.iter().chain(extra_rom_dirs.iter());
    let separator = ";"; // MAME uses semicolon on all platforms
    let rom_paths = all_dirs
    .map(|p| p.to_string_lossy())
    .collect::<Vec<_>>()
    .join(separator);

    cmd.arg("-rompath").arg(&rom_paths);

    // Apply graphics settings
    let preset = graphics_config.get_game_preset(rom_name);
    for arg in preset.to_mame_args() {
        cmd.arg(arg);
    }

    // APPLY VIDEO SETTINGS
    if video_settings.video_backend != "auto" {
        cmd.arg("-video").arg(&video_settings.video_backend);
    }

    if video_settings.window_mode {
        cmd.arg("-window");
    }

    if video_settings.maximize {
        cmd.arg("-maximize");
    }

    if video_settings.wait_vsync {
        cmd.arg("-waitvsync");
    }

    if video_settings.sync_refresh {
        cmd.arg("-syncrefresh");
    }

    if video_settings.prescale > 0 {
        cmd.arg("-prescale").arg(video_settings.prescale.to_string());
    }

    if !video_settings.keep_aspect {
        cmd.arg("-nokeepaspect");
    }

    if !video_settings.filter {
        cmd.arg("-nofilter");
    }

    if video_settings.num_screens > 1 {
        cmd.arg("-numscreens").arg(video_settings.num_screens.to_string());
    }

    // Add custom arguments if any
    if !video_settings.custom_args.is_empty() {
        for arg in video_settings.custom_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    // ROM name must be last
    cmd.arg(rom_name);

    // Spawn and return the child process
    let child = cmd.spawn()?;

    Ok(child)
}

// UPDATED FUNCTION WITH VIDEO SETTINGS
pub fn launch_rom_with_mame(
    rom: &str,
    rom_dirs: &[PathBuf],
    _extra_dirs: &[PathBuf],
    mame_executable: &str,
    graphics_config: &GraphicsConfig,
    video_settings: &VideoSettings,  // ADD THIS PARAMETER
) {
    eprintln!("=== LAUNCH DEBUG ===");
    eprintln!("Launching ROM: {} with MAME: {}", rom, mame_executable);
    eprintln!("ROM directories count: {}", rom_dirs.len());

    // Print each ROM directory
    for (i, dir) in rom_dirs.iter().enumerate() {
        eprintln!("  ROM dir {}: {:?}", i, dir);
    }

    // Only use ROM directories for the rompath
    // MAME uses semicolon as separator on all platforms
    let separator = ";";
    let rom_paths = rom_dirs
    .iter()
    .map(|p| p.to_string_lossy())
    .collect::<Vec<_>>()
    .join(separator);

    eprintln!("Combined ROM paths: {}", rom_paths);

    // Get MAME data directory
    let mame_data_dir = get_mame_data_dir();

    // Create directories if they don't exist
    let nvram_dir = mame_data_dir.join("nvram");
    let cfg_dir = mame_data_dir.join("cfg");
    let state_dir = mame_data_dir.join("sta");
    let snap_dir = mame_data_dir.join("snap");

    let _ = std::fs::create_dir_all(&nvram_dir);
    let _ = std::fs::create_dir_all(&cfg_dir);
    let _ = std::fs::create_dir_all(&state_dir);
    let _ = std::fs::create_dir_all(&snap_dir);

    // Build the command
    let mut cmd = Command::new(mame_executable);

    // MOST RELIABLE: Set the working directory to our MAME data directory
    // This ensures MAME creates its folders in the right place
    cmd.current_dir(&mame_data_dir);

    // Add rompath and ROM name
    // When running from the data directory, MAME will use relative paths
    cmd.arg("-rompath").arg(&rom_paths)
    .arg("-nvram_directory").arg("nvram")
    .arg("-cfg_directory").arg("cfg")
    .arg("-state_directory").arg("sta")
    .arg("-snapshot_directory").arg("snap");

    // ADD GRAPHICS PRESET ARGUMENTS HERE
    let preset = graphics_config.get_game_preset(rom);
    eprintln!("Using graphics preset: {}", preset.name);
    for arg in preset.to_mame_args() {
        eprintln!("  Graphics arg: {}", arg);
        cmd.arg(arg);
    }

    // APPLY VIDEO SETTINGS
    eprintln!("Applying video settings:");
    if video_settings.video_backend != "auto" {
        eprintln!("  Video backend: {}", video_settings.video_backend);
        cmd.arg("-video").arg(&video_settings.video_backend);
    }

    if video_settings.window_mode {
        eprintln!("  Window mode enabled");
        cmd.arg("-window");
    }

    if video_settings.maximize {
        eprintln!("  Maximize enabled");
        cmd.arg("-maximize");
    }

    if video_settings.wait_vsync {
        eprintln!("  V-Sync enabled");
        cmd.arg("-waitvsync");
    }

    if video_settings.sync_refresh {
        eprintln!("  Sync refresh enabled");
        cmd.arg("-syncrefresh");
    }

    if video_settings.prescale > 0 {
        eprintln!("  Prescale: {}x", video_settings.prescale);
        cmd.arg("-prescale").arg(video_settings.prescale.to_string());
    }

    if !video_settings.keep_aspect {
        eprintln!("  Keep aspect disabled");
        cmd.arg("-nokeepaspect");
    }

    if !video_settings.filter {
        eprintln!("  Filter disabled");
        cmd.arg("-nofilter");
    }

    if video_settings.num_screens > 1 {
        eprintln!("  Number of screens: {}", video_settings.num_screens);
        cmd.arg("-numscreens").arg(video_settings.num_screens.to_string());
    }

    // Add custom arguments if any
    if !video_settings.custom_args.is_empty() {
        eprintln!("  Custom args: {}", video_settings.custom_args);
        for arg in video_settings.custom_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    // ROM name must be last
    cmd.arg(rom);

    eprintln!("Working directory: {:?}", mame_data_dir);
    eprintln!("Full command: cd {:?} && {:?} -rompath {} -nvram_directory nvram -cfg_directory cfg ... {}",
              mame_data_dir, mame_executable, rom_paths, rom);
    eprintln!("===================");

    // Spawn and check for errors
    match cmd.spawn() {
        Ok(child) => {
            eprintln!("MAME launched successfully with PID: {:?}", child.id());
        },
        Err(e) => {
            eprintln!("Failed to launch MAME: {}", e);
        }
    }
}
