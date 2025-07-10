use eframe::egui;
use crate::app::MyApp;

pub fn show_mame_status(app: &MyApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if app.config.mame_executables.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(255, 200, 100),
                             "No MAME executables configured. Use File → MAME Executables Manager to add MAME versions.");
        } else {
            ui.label("Active MAME:");
            if let Some(current_mame) = app.config.mame_executables.get(app.config.selected_mame_index) {
                ui.label(&current_mame.name);
                ui.small_button("📁").on_hover_text("Manage in File menu");

                if app.config.mame_executables.len() > 1 {
                    ui.separator();
                    ui.label(format!("({} versions configured)", app.config.mame_executables.len()));
                }
            }
        }
    });
}

pub fn show_path_status(app: &MyApp, ui: &mut egui::Ui) {
    if !app.config.rom_dirs.is_empty() || !app.config.extra_asset_dirs.is_empty() {
        ui.group(|ui| {
            ui.label("Configured Paths:");

            // Show ROM paths (which include ROMs, CHDs, and BIOS)
            if !app.config.rom_dirs.is_empty() {
                ui.label("ROM Paths (ROMs/CHDs/BIOS):");
                for (i, dir) in app.config.rom_dirs.iter().enumerate() {
                    ui.label(format!("  {}. {}", i + 1, dir.display()));
                }
            }

            // Show Extras paths
            if !app.config.extra_asset_dirs.is_empty() {
                ui.label("Extras Paths (Artwork/Snapshots):");
                for (i, dir) in app.config.extra_asset_dirs.iter().enumerate() {
                    ui.label(format!("  {}. {}", i + 1, dir.display()));
                }
            }

            // Show icon path if configured
            if app.config.show_rom_icons {
                let mut found_icons = false;
                for asset_dir in &app.config.extra_asset_dirs {
                    let icons_path = asset_dir.join("icons");
                    if icons_path.exists() && icons_path.is_dir() {
                        ui.label(format!("Icons Path: {}", icons_path.display()));
                        found_icons = true;
                        break;
                    }
                }

                if !found_icons {
                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100),
                                     "Icons enabled but no 'icons' folder found in Extra Asset directories");
                }
            }
        });
    } else if !app.config.mame_executables.is_empty() {
        ui.colored_label(egui::Color32::from_rgb(255, 200, 100),
                         "No paths configured. Use Options → Directories to add paths.");
    }
}
