use eframe::egui;
use crate::app::MyApp;
use crate::models::RomSetType;
use crate::rom_utils::apply_rom_filters;

pub fn show_status_bar(app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Filter status
            let total_roms = app.roms.len();

            // Calculate filtered count using the same logic as the main filter
            let filtered_roms_count = app.roms.iter()
            .filter(|(display, filename)| apply_rom_filters(&app.config.filter_settings, &app.game_metadata, display, filename, &app.config.favorite_games))
            .count();

            let status_text = if app.config.filter_settings.search_text.is_empty() &&
            app.config.filter_settings.year_from.is_empty() &&
            app.config.filter_settings.year_to.is_empty() &&
            app.config.filter_settings.manufacturer.is_empty() {
                format!("Showing {} games in collection", total_roms)
            } else {
                format!("Filtered: {} games", filtered_roms_count)
            };
            ui.label(status_text);

            ui.separator();

            // MAME statistics
            if app.total_games_count > 0 {
                ui.label(format!("{} | Total supported: {} games ({} working, {}% playable)",
                                 app.mame_version,
                                 app.total_games_count,
                                 app.working_games_count,
                                 (app.working_games_count as f32 / app.total_games_count as f32 * 100.0) as u32));
            } else if !app.mame_version.is_empty() {
                ui.label(&app.mame_version);
            } else {
                ui.label("No MAME executable configured");
            }

            ui.separator();

            // ROM set type indicator
            let rom_set_type = app.get_rom_set_type();
            let (type_icon, type_color) = match rom_set_type {
                RomSetType::Merged => ("📦", egui::Color32::from_rgb(200, 200, 200)),
                      RomSetType::Split => ("🔗", egui::Color32::from_rgb(200, 200, 200)),
                      RomSetType::NonMerged => ("📁", egui::Color32::from_rgb(200, 200, 200)),
                      RomSetType::Unknown => ("❓", egui::Color32::from_rgb(255, 200, 100)),
            };

            let type_name = match rom_set_type {
                RomSetType::Merged => {
                    if !app.has_audit_file() && app.config.use_mame_audit {
                        "Merged ROMs (no audit!)".to_string()
                    } else if app.config.use_mame_audit {
                        if let Some(current_mame) = app.config.mame_executables.get(app.config.selected_mame_index) {
                            let mame_id = app.get_mame_identifier(current_mame);
                            if let Some(last_audit) = app.config.mame_audit_times.get(&mame_id) {
                                format!("Merged ROMs (audited: {})", last_audit)
                            } else {
                                "Merged ROMs (no audit for this MAME)".to_string()
                            }
                        } else {
                            "Merged ROMs".to_string()
                        }
                    } else {
                        "Merged ROMs (audit disabled)".to_string()
                    }
                }
                RomSetType::Split => "Split ROMs".to_string(),
                      RomSetType::NonMerged => "Non-Merged ROMs".to_string(),
                      RomSetType::Unknown => "Unknown ROM Set".to_string(),
            };

            let final_color = if matches!(rom_set_type, RomSetType::Merged) && !app.has_audit_file() && app.config.use_mame_audit {
                egui::Color32::from_rgb(255, 100, 100)
            } else if matches!(rom_set_type, RomSetType::Merged) && app.config.use_mame_audit && app.has_audit_file() {
                egui::Color32::from_rgb(100, 255, 100)
            } else if matches!(rom_set_type, RomSetType::Merged) && !app.config.use_mame_audit {
                egui::Color32::from_rgb(255, 200, 100)
            } else {
                type_color
            };

            ui.colored_label(final_color, format!("{} {}", type_icon, type_name))
            .on_hover_text("Click Help → ROM Set Information for details");

            // Add space to right-align the next items
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Improved loading indicators
                if app.roms_loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                }

                if app.audit_in_progress {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Auditing...");
                    });
                }
            });
        });
    });
}
