mod status;
mod filters;
mod table_header;
mod table;
mod table_row;
mod sorting;
mod launch;

use eframe::egui;
use crate::app::MyApp;

// pub use sorting::{sort_rom_list, SortableRom};

pub fn show_rom_list(app: &mut MyApp, ctx: &egui::Context) {
    // Process icon loading queue
    app.process_icon_queue(ctx);

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("RMAMEUI");

        // Show current MAME executable(s)
        status::show_mame_status(app, ui);

        // Show configured paths
        status::show_path_status(app, ui);

        // Warning for merged ROMs without audit
        if app.config.use_mame_audit && app.roms.is_empty() && !app.config.rom_dirs.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(255, 200, 100),
                             "Using merged ROMs - Please run 'ROM Audit' from Options menu first!");
        }

        ui.separator();

        // Search and Filter Section
        filters::show_filters(app, ui);

        ui.separator();

        // Table Header
        table_header::show_table_header(app, ui);

        // ROM list display
        table::show_rom_table(app, ui, ctx);

        ui.separator();

        // Check for loaded ROMs
        if let Some(rx) = &app.roms_tx {
            if let Ok(loaded_roms) = rx.try_recv() {
                println!("\n=== ROMS LOADED ===");
                println!("Total ROMs loaded: {}", loaded_roms.len());

                // Show examples of what was loaded
                println!("First 10 ROMs loaded:");
                for (display, name) in loaded_roms.iter().take(10) {
                    println!("  {} [{}]", display, name);
                }

                app.roms = loaded_roms;
                app.roms_loading = false;
                app.roms_tx = None;
            }
        }

        // Launch button
        launch::show_launch_button(app, ui);
    });
}
