use eframe::egui;
use crate::app::MyApp;
use crate::models::{FilterSettings, Theme};
use crate::config::get_mame_data_dir;
use std::process::Command;

pub fn show_menu_bar(app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File Menu
            ui.menu_button("File", |ui| {
                if ui.button("MAME Executables Manager...").clicked() {
                    app.show_mame_manager = true;
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Save Settings").clicked() {
                    app.save_config();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    app.save_config();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // Options Menu
            ui.menu_button("Options", |ui| {
                ui.menu_button("Directories", |ui| {
                    if ui.button("Add ROM Path...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            app.config.rom_dirs.push(path.clone());
                            app.config.extra_rom_dirs.push(path); // Same path for ROMs, CHDs, and BIOS
                            app.save_config();

                            // Reload ROMs if we have at least one MAME configured
                            app.reload_roms();
                        }
                        ui.close_menu();
                    }

                    if ui.button("Add Extras Path...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            app.config.extra_asset_dirs.push(path);
                            app.save_config();
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Open MAME Data Folder").clicked() {
                        let mame_data_dir = get_mame_data_dir();
                        // Open the folder in the system's file manager
                        #[cfg(target_os = "linux")]
                        let _ = Command::new("xdg-open").arg(&mame_data_dir).spawn();
                        #[cfg(target_os = "windows")]
                        let _ = Command::new("explorer").arg(&mame_data_dir).spawn();
                        #[cfg(target_os = "macos")]
                        let _ = Command::new("open").arg(&mame_data_dir).spawn();
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Clear All Paths").clicked() {
                        app.config.rom_dirs.clear();
                        app.config.extra_rom_dirs.clear();
                        app.config.extra_asset_dirs.clear();
                        app.roms.clear();
                        app.config.selected_rom = None;
                        app.screenshot = None;
                        app.config.filter_settings = FilterSettings::default();
                        app.all_manufacturers.clear();
                        app.art_texture = None;
                        app.save_config();
                        ui.close_menu();
                    }
                });

                ui.separator();

                // ADD VIDEO SETTINGS MENU ITEM HERE
                if ui.button("🖥️ Video Settings...").clicked() {
                    app.show_video_settings = true;
                    ui.close_menu();
                }

                ui.separator();

                ui.menu_button("🎨 Theme", |ui| {
                    for theme in Theme::all() {
                        if ui.radio(app.config.theme == theme, theme.name()).clicked() {
                            app.config.theme = theme;
                            app.save_config();
                            ui.close_menu();
                        }
                    }
                });

                ui.separator();

                ui.menu_button("ROM Audit", |ui| {
                    ui.label("For merged ROM sets:");
                    ui.separator();

                    // Show current MAME info
                    if let Some(current_mame) = app.config.mame_executables.get(app.config.selected_mame_index) {
                        ui.label(format!("Current: {}", current_mame.name));

                        let mame_id = app.get_mame_identifier(current_mame);
                        if let Some(last_audit) = app.config.mame_audit_times.get(&mame_id) {
                            ui.label(format!("Last audit: {}", last_audit));
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "Never audited");
                        }

                        ui.separator();
                    }

                    if ui.button("🔍 Run ROM Audit").clicked() {
                        app.run_mame_audit();
                        ui.close_menu();
                    }

                    ui.separator();

                    let was_checked = app.config.use_mame_audit;
                    if ui.checkbox(&mut app.config.use_mame_audit, "Use audit data")
                        .on_hover_text("Enable to support merged ROM sets after running audit")
                        .changed() {
                            if app.config.use_mame_audit != was_checked {
                                app.save_config();
                                app.reload_roms(); // Reload with new setting
                            }
                        }

                        ui.separator();
                    ui.label("ℹ Audit scans inside ZIP files");
                    ui.label("to find clones in merged sets.");
                    ui.label("Each MAME version has its own audit.");
                });

                ui.separator();

                ui.menu_button("Maintenance", |ui| {
                    if ui.button("Clean up orphaned audit files").clicked() {
                        app.cleanup_orphaned_audit_files();
                        ui.close_menu();
                    }

                    if ui.button("View audit files").clicked() {
                        let audit_files = app.list_audit_files();
                        println!("Audit files found: {:?}", audit_files);
                        ui.close_menu();
                    }
                });
            });

            // Help Menu
            ui.menu_button("Help", |ui| {
                if ui.button("ROM Set Information").clicked() {
                    app.show_rom_set_info = true;
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Debug Info").clicked() {
                    app.show_debug = true;
                    ui.close_menu();
                }
                if ui.button("ROM Diagnostics").clicked() {
                    app.show_rom_diagnostics = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("About").clicked() {
                    app.show_about = true;
                    ui.close_menu();
                }

                // Add debug menu items
                ui.separator();

                if ui.button("Debug Parent/Clone Info").clicked() {
                    app.debug_parent_clone_relationships();
                }

                if ui.button("ROM Loading Debug Info").clicked() {
                    app.show_debug = true;
                }

                if ui.button("ROM Setup Diagnostics").clicked() {
                    app.show_rom_diagnostics = true;
                }
            });
        });
    });
}
