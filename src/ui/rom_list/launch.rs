use eframe::egui;
use crate::app::MyApp;
use crate::mame_utils::launch_rom_with_mame_tracked;

pub fn show_launch_button(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Launch Selected ROM").clicked() {
            launch_selected_rom(app);
        }

        if let Some(selected) = &app.config.selected_rom {
            ui.label(format!("Selected: {}", selected));
        }

        show_loading_status(app, ui);
        show_icon_loading_status(app, ui);
        show_running_games_status(app, ui);
    });
}

fn launch_selected_rom(app: &mut MyApp) {
    if let Some(rom_name) = &app.config.selected_rom {
        if !app.config.mame_executables.is_empty() {
            let mame_idx = app.config.game_preferred_mame.get(rom_name)
                .copied()
                .unwrap_or(app.config.selected_mame_index);

            if let Some(mame) = app.config.mame_executables.get(mame_idx) {
                match launch_rom_with_mame_tracked(
                    rom_name,
                    &app.config.rom_dirs,
                    &app.config.extra_rom_dirs,
                    &mame.path,
                    &app.config.graphics_config,
                    &app.config.video_settings  // ADD THIS LINE
                ) {
                    Ok(child) => {
                        app.running_games.insert(rom_name.clone(), (child, std::time::Instant::now()));
                        println!("Started tracking game: {}", rom_name);
                    }
                    Err(e) => {
                        println!("Failed to launch game: {}", e);
                    }
                }
            }
        }
    }
}

fn show_loading_status(app: &MyApp, ui: &mut egui::Ui) {
    if app.roms_loading {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Loading ROM collection...");
        });
    }
}

fn show_icon_loading_status(app: &MyApp, ui: &mut egui::Ui) {
    if app.config.show_rom_icons && !app.icon_load_queue.is_empty() {
        ui.separator();
        ui.label(format!("Loading icons: {} queued", app.icon_load_queue.len()));
    }
}

fn show_running_games_status(app: &MyApp, ui: &mut egui::Ui) {
    if !app.running_games.is_empty() {
        ui.separator();
        ui.label(format!("Games running: {}", app.running_games.len()));
    }
}
