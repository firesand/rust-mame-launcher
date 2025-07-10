use eframe::egui;
use crate::app::MyApp;
use crate::models::{MameExecutable, RomSetType, VideoSettings, AppConfig};
use crate::mame_utils::{get_mame_version, load_mame_metadata_parallel_with_exec};
use chrono;

pub fn show_dialogs(app: &mut MyApp, ctx: &egui::Context) {
    show_close_dialog(app, ctx);
    show_audit_progress_dialog(app, ctx);
    show_mame_manager_dialog(app, ctx);
    show_about_dialog(app, ctx);
    show_debug_window(app, ctx);
    show_rom_diagnostics_window(app, ctx);
    show_rom_set_info_dialog(app, ctx);
    show_context_menu(app, ctx);
    show_video_settings_dialog(app, ctx);  // ADD THIS LINE
}

// FIXED VERSION OF show_video_settings_dialog
pub fn show_video_settings_dialog(app: &mut MyApp, ctx: &egui::Context) {
    let mut show_dialog = app.show_video_settings;

    if show_dialog {
        egui::Window::new("Video Settings")
        .open(&mut show_dialog)
        .resizable(false)
        .default_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("OSD Video Options");
            ui.separator();

            // Video backend selection
            ui.horizontal(|ui| {
                ui.label("Video Backend:");
                egui::ComboBox::from_label("")
                .selected_text(&app.config.video_settings.video_backend)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.config.video_settings.video_backend, "auto".to_string(), "Auto");
                    ui.selectable_value(&mut app.config.video_settings.video_backend, "soft".to_string(), "Software");
                    ui.selectable_value(&mut app.config.video_settings.video_backend, "opengl".to_string(), "OpenGL");
                    #[cfg(target_os = "windows")]
                    ui.selectable_value(&mut app.config.video_settings.video_backend, "d3d".to_string(), "Direct3D");
                    ui.selectable_value(&mut app.config.video_settings.video_backend, "bgfx".to_string(), "BGFX");
                });
            });

            ui.add_space(10.0);

            // Window options
            ui.group(|ui| {
                ui.label("Window Options:");
                ui.checkbox(&mut app.config.video_settings.window_mode, "Run in window");
                ui.checkbox(&mut app.config.video_settings.maximize, "Start maximized");

                ui.horizontal(|ui| {
                    ui.label("Number of screens:");
                    ui.add(egui::Slider::new(&mut app.config.video_settings.num_screens, 1..=4));
                });
            });

            ui.add_space(10.0);

            // Performance options
            ui.group(|ui| {
                ui.label("Performance Options:");
                ui.checkbox(&mut app.config.video_settings.wait_vsync, "Wait for V-Sync");
                ui.checkbox(&mut app.config.video_settings.sync_refresh, "Sync to monitor refresh");

                ui.horizontal(|ui| {
                    ui.label("Prescale:");
                    ui.add(egui::Slider::new(&mut app.config.video_settings.prescale, 0..=3)
                    .text("x")
                    .clamping(egui::SliderClamping::Always));  // FIXED: Use clamping instead of clamp_to_range
                });
                if app.config.video_settings.prescale > 0 {
                    ui.label("  (Scales rendering before filters)");
                }
            });

            ui.add_space(10.0);

            // Display options
            ui.group(|ui| {
                ui.label("Display Options:");
                ui.checkbox(&mut app.config.video_settings.keep_aspect, "Keep aspect ratio");
                ui.checkbox(&mut app.config.video_settings.filter, "Bilinear filtering");
            });

            ui.add_space(10.0);

            // Custom arguments
            ui.label("Custom arguments:");
            ui.text_edit_singleline(&mut app.config.video_settings.custom_args);
            ui.label("(Additional MAME command-line arguments)");

            ui.add_space(20.0);

            // Buttons
            ui.horizontal(|ui| {
                if ui.button("Reset to Defaults").clicked() {
                    app.config.video_settings = VideoSettings::default();
                }
            });
        });

        app.show_video_settings = show_dialog;

        // Save config when dialog is closed
        if !app.show_video_settings {
            app.save_config();
        }
    }
}

// Replace the video_settings_dialog function with this version:

pub fn video_settings_dialog(
    ctx: &egui::Context,
    show_video_settings: &mut bool,
    video_settings: &mut VideoSettings,
    _config: &AppConfig,
) {
    let mut should_close = false;  // Track if we should close the dialog

    egui::Window::new("Video Settings")
    .open(show_video_settings)  // Pass the reference directly
    .resizable(false)
    .default_width(400.0)
    .show(ctx, |ui| {
        ui.heading("OSD Video Options");
        ui.separator();

        // Video backend selection
        ui.horizontal(|ui| {
            ui.label("Video Backend:");
            egui::ComboBox::from_label("")
            .selected_text(&video_settings.video_backend)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut video_settings.video_backend, "auto".to_string(), "Auto");
                ui.selectable_value(&mut video_settings.video_backend, "soft".to_string(), "Software");
                ui.selectable_value(&mut video_settings.video_backend, "opengl".to_string(), "OpenGL");
                #[cfg(target_os = "windows")]
                ui.selectable_value(&mut video_settings.video_backend, "d3d".to_string(), "Direct3D");
                ui.selectable_value(&mut video_settings.video_backend, "bgfx".to_string(), "BGFX");
            });
        });

        ui.add_space(10.0);

        // Window options
        ui.group(|ui| {
            ui.label("Window Options:");
            ui.checkbox(&mut video_settings.window_mode, "Run in window");
            ui.checkbox(&mut video_settings.maximize, "Start maximized");

            ui.horizontal(|ui| {
                ui.label("Number of screens:");
                ui.add(egui::Slider::new(&mut video_settings.num_screens, 1..=4));
            });
        });

        ui.add_space(10.0);

        // Performance options
        ui.group(|ui| {
            ui.label("Performance Options:");
            ui.checkbox(&mut video_settings.wait_vsync, "Wait for V-Sync");
            ui.checkbox(&mut video_settings.sync_refresh, "Sync to monitor refresh");

            ui.horizontal(|ui| {
                ui.label("Prescale:");
                ui.add(egui::Slider::new(&mut video_settings.prescale, 0..=3)
                .text("x")
                .clamping(egui::SliderClamping::Always));
            });
            if video_settings.prescale > 0 {
                ui.label("  (Scales rendering before filters)");
            }
        });

        ui.add_space(10.0);

        // Display options
        ui.group(|ui| {
            ui.label("Display Options:");
            ui.checkbox(&mut video_settings.keep_aspect, "Keep aspect ratio");
            ui.checkbox(&mut video_settings.filter, "Bilinear filtering");
        });

        ui.add_space(10.0);

        // Custom arguments
        ui.label("Custom arguments:");
        ui.text_edit_singleline(&mut video_settings.custom_args);
        ui.label("(Additional MAME command-line arguments)");

        ui.add_space(20.0);

        // Buttons
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                should_close = true;
            }

            if ui.button("Cancel").clicked() {
                should_close = true;
            }

            if ui.button("Reset to Defaults").clicked() {
                *video_settings = VideoSettings::default();
            }
        });
    });

    // Handle closing after the window is shown
    if should_close {
        *show_video_settings = false;
    }
}

fn show_close_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_close_dialog {
        egui::Window::new("Confirm Exit")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("What would you like to do?");
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.button("🚪 Quit Application").clicked() {
                        app.save_config();
                        app.pending_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.add_space(10.0);

                    if ui.button("🔽 Minimize to Taskbar").clicked() {
                        app.show_close_dialog = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    ui.add_space(10.0);

                    if ui.button("❌ Cancel").clicked() {
                        app.show_close_dialog = false;
                    }
                });
            });
        });
    }
}

fn show_audit_progress_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.audit_in_progress {
        egui::Window::new("ROM Audit Progress")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Auditing ROM Collection");
            ui.separator();

            // Show current progress text
            ui.label(&app.audit_progress);

            // Add a spinning indicator with more visual prominence
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Please wait...");
            });

            // Add a progress bar (if you can estimate progress)
            let progress = if app.audit_progress.contains("complete") {
                1.0
            } else if app.audit_progress.contains("scanning") {
                0.5
            } else {
                0.1
            };
            ui.add_space(10.0);
            ui.add(egui::ProgressBar::new(progress).show_percentage());

            ui.add_space(10.0);
            ui.label("This may take several minutes for large collections...");
            ui.label("MAME is scanning inside all ROM files.");

            // Add a cancel button
            ui.add_space(10.0);
            if ui.button("❌ Cancel Audit").clicked() {
                app.audit_in_progress = false;
                app.audit_tx = None;
                app.audit_start_time = None;
            }
        });

        // Check for audit updates
        if let Some(rx) = &app.audit_tx {
            if let Ok(msg) = rx.try_recv() {
                if msg == "AUDIT_COMPLETE" {
                    app.audit_in_progress = false;
                    app.audit_tx = None;
                    app.audit_start_time = None;
                    app.config.use_mame_audit = true;

                    // Update per-MAME audit time
                    if let Some(current_mame) = app.config.mame_executables.get(app.config.selected_mame_index) {
                        let mame_id = app.get_mame_identifier(current_mame);
                        let audit_time = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
                        app.config.mame_audit_times.insert(mame_id, audit_time);
                    }

                    app.save_config();
                    // Reload ROMs with audit data
                    app.reload_roms();
                } else if msg == "AUDIT_FAILED" {
                    app.audit_in_progress = false;
                    app.audit_tx = None;
                    app.audit_start_time = None;
                } else {
                    app.audit_progress = msg;
                }
            }
        }
    }
}

fn show_mame_manager_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_mame_manager {
        egui::Window::new("MAME Executables Manager")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Manage MAME Versions");
            ui.separator();

            // Add new MAME button
            if ui.button("➕ Add MAME Executable").clicked() {
                let file_dialog = if cfg!(target_os = "linux") {
                    rfd::FileDialog::new().set_file_name("mame")
                } else if cfg!(target_os = "windows") {
                    rfd::FileDialog::new().add_filter("Executable", &["exe"])
                } else {
                    rfd::FileDialog::new()
                };

                if let Some(path) = file_dialog.pick_file() {
                    if let Some(path_str) = path.to_str() {
                        let version = get_mame_version(path_str);
                        let name = format!("MAME {}", version.split_whitespace().nth(1).unwrap_or("Unknown"));

                        // Load metadata for this MAME
                        let metadata = load_mame_metadata_parallel_with_exec(path_str);
                        let total = metadata.iter()
                        .filter(|(_, meta)| !meta.is_device && !meta.is_bios)
                        .count();
                        let working = metadata.iter()
                        .filter(|(_, meta)| !meta.is_device && !meta.is_bios && !meta.is_mechanical && !meta.runnable)
                        .count();

                        app.config.mame_executables.push(MameExecutable {
                            name,
                            path: path_str.to_string(),
                                                         version,
                                                         total_games: total,
                                                         working_games: working,
                        });

                        // If this is the first MAME, load its metadata
                        if app.config.mame_executables.len() == 1 {
                            app.config.selected_mame_index = 0;
                            app.game_metadata = metadata;
                            app.mame_titles = app.game_metadata.iter().map(|(k, v)| (k.clone(), v.description.clone())).collect();
                            app.total_games_count = total;
                            app.working_games_count = working;
                            app.mame_version = app.config.mame_executables[0].version.clone();
                        }

                        app.save_config();
                    }
                }
            }

            ui.separator();

            // List existing MAME executables
            if app.config.mame_executables.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100),
                                 "No MAME executables configured. Click 'Add MAME Executable' to add one.");
            } else {
                ui.label("Configured MAME versions:");
                ui.add_space(10.0);

                let mut to_remove = None;
                let mut to_select = None;

                for (idx, mame) in app.config.mame_executables.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Radio button for default selection
                            if ui.radio(app.config.selected_mame_index == idx, &mame.name).clicked() {
                                to_select = Some(idx);
                            }

                            ui.separator();

                            ui.vertical(|ui| {
                                ui.label(format!("Path: {}", mame.path));
                                ui.label(format!("Version: {}", mame.version));
                                ui.label(format!("Games: {} total, {} working", mame.total_games, mame.working_games));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("❌ Remove").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        });
                    });
                    ui.add_space(5.0);
                }

                // Handle removal
                if let Some(idx) = to_remove {
                    // Clean up the audit file for this MAME
                    app.cleanup_audit_file(idx);

                    app.config.mame_executables.remove(idx);
                    if app.config.selected_mame_index >= app.config.mame_executables.len() && !app.config.mame_executables.is_empty() {
                        app.config.selected_mame_index = app.config.mame_executables.len() - 1;
                    }
                    app.save_config();
                }

                // Handle selection change
                if let Some(idx) = to_select {
                    app.config.selected_mame_index = idx;
                    let mame_path = app.config.mame_executables[idx].path.clone();
                    app.load_mame_data(&mame_path);
                    app.reload_roms();
                    app.save_config();
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    app.show_mame_manager = false;
                }
            });
        });
    }
}

fn show_about_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_about {
        egui::Window::new("About")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rust MAME Launcher");
                ui.add_space(10.0);
                ui.label("Version 0.2.0");
                ui.add_space(10.0);
                ui.label("A MAME frontend built with Rust");
                ui.add_space(5.0);
                ui.label("Created by Edo Hikmahtiar - Indonesia");
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label("Features:");
                ui.label("• Multi-folder ROM management");
                ui.label("• Multiple MAME version support");
                ui.label("• Game artwork display");
                ui.label("• Advanced search and filtering");
                ui.label("• MAME metadata integration");
                ui.label("• Automatic settings persistence");
                ui.label("• Organized MAME data storage");
                ui.label("• Parent/Clone ROM grouping");
                ui.label("• Merged ROM set support");
                ui.label("• OSD Video Settings");  // ADD THIS LINE
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                ui.label("MAME data files are stored in:");
                ui.code("~/.mame/rust-mame-launcher/");
                ui.add_space(20.0);
                if ui.button("Close").clicked() {
                    app.show_about = false;
                }
            });
        });
    }
}

fn show_debug_window(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_debug {
        egui::Window::new("Debug Information")
        .collapsible(true)
        .resizable(true)
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.heading("ROM Loading Debug Information");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.monospace(app.debug_rom_loading());
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    // Force refresh
                }
                if ui.button("Force Reload ROMs").clicked() {
                    app.reload_roms();
                }
                if ui.button("Close").clicked() {
                    app.show_debug = false;
                }
            });
        });
    }
}

fn show_rom_diagnostics_window(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_rom_diagnostics {
        egui::Window::new("ROM Setup Diagnostics")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.heading("ROM Setup Analysis");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.monospace(app.diagnose_rom_setup());
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Run Audit Now").clicked() {
                    app.run_mame_audit();
                    app.show_rom_diagnostics = false;
                }
                if ui.button("Close").clicked() {
                    app.show_rom_diagnostics = false;
                }
            });
        });
    }
}

fn show_rom_set_info_dialog(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_rom_set_info {
        egui::Window::new("ROM Set Analysis")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("ROM Collection Analysis");
            ui.separator();

            // Detect ROM set type
            let rom_set_type = app.get_rom_set_type();

            // Count statistics
            let total_roms = app.roms.len();
            let parent_count = app.roms.iter()
            .filter(|(_, name)| {
                app.game_metadata.get(name)
                .map(|m| !m.is_clone)
                .unwrap_or(false)
            })
            .count();
            let clone_count = total_roms - parent_count;

            // Basic statistics
            ui.group(|ui| {
                ui.label(egui::RichText::new("Collection Statistics").strong());
                ui.horizontal(|ui| {
                    ui.label("Total ROM files:");
                    ui.label(total_roms.to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("Parent ROMs:");
                    ui.label(parent_count.to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("Clone ROMs:");
                    ui.label(clone_count.to_string());
                });
            });

            ui.add_space(10.0);

            // ROM set type detection
            ui.group(|ui| {
                ui.label(egui::RichText::new("Detected ROM Set Type").strong());

                let (type_icon, type_name, type_desc) = match rom_set_type {
                    RomSetType::Merged => (
                        "📦",
                        "MERGED ROM SET",
                        "Parent ZIP files contain both parent and clone ROMs"
                    ),
                    RomSetType::Split => (
                        "🔗",
                        "SPLIT ROM SET",
                        "Clone ZIPs contain only differences, require parent ROMs"
                    ),
                    RomSetType::NonMerged => (
                        "📁",
                        "NON-MERGED ROM SET",
                        "Each game ZIP contains all required ROMs"
                    ),
                    RomSetType::Unknown => (
                        "❓",
                        "UNKNOWN/MIXED",
                        "Cannot determine ROM set type"
                    ),
                };

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{} {}", type_icon, type_name))
                    .size(16.0)
                    .strong());
                });
                ui.label(type_desc);
            });

            ui.add_space(10.0);

            // Recommendations
            ui.group(|ui| {
                ui.label(egui::RichText::new("Recommendations").strong());

                match rom_set_type {
                    RomSetType::Merged => {
                        ui.label("✅ Your ROM set is MERGED");
                        ui.label("• Clone games are stored inside parent ZIP files");
                        ui.label("• You MUST run 'ROM Audit' to see all games");
                        ui.label("• Without audit, only parent games will be visible");
                        ui.separator();
                        if !app.config.use_mame_audit {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 100),
                                             "⚠ Enable 'Use audit data' in Options → ROM Audit"
                            );
                        }
                        if !app.has_audit_file() {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 100, 100),
                                             "❌ No audit file found - Run ROM Audit now!"
                            );
                            if ui.button("🔍 Run ROM Audit").clicked() {
                                app.run_mame_audit();
                                app.show_rom_set_info = false;
                            }
                        }
                    }
                    RomSetType::Split => {
                        ui.label("✅ Your ROM set is SPLIT");
                        ui.label("• Clone games depend on parent ROMs");
                        ui.label("• Missing parent ROMs will prevent clones from working");

                        let missing = app.get_missing_parent_roms();
                        if !missing.is_empty() {
                            ui.separator();
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 100, 100),
                                             format!("⚠ {} clone(s) missing parent ROMs:", missing.len())
                            );

                            egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for (clone, parent) in missing.iter().take(20) {
                                    ui.label(format!("  {} needs {}", clone, parent));
                                }
                                if missing.len() > 20 {
                                    ui.label(format!("  ... and {} more", missing.len() - 20));
                                }
                            });
                        }
                    }
                    RomSetType::NonMerged => {
                        ui.label("✅ Your ROM set is NON-MERGED");
                        ui.label("• Each game ZIP contains all required files");
                        ui.label("• No special handling needed");
                        ui.label("• Uses more disk space but simpler to manage");
                    }
                    RomSetType::Unknown => {
                        ui.label("❓ Cannot determine ROM set type");
                        ui.label("• You may have a mixed or incomplete set");
                        ui.label("• Try running ROM Audit for better detection");
                    }
                }
            });

            ui.add_space(10.0);

            // Storage efficiency
            ui.group(|ui| {
                ui.label(egui::RichText::new("Storage Efficiency").strong());
                ui.label(match rom_set_type {
                    RomSetType::Merged => "🟢 Most efficient - no duplicate data",
                    RomSetType::Split => "🟡 Moderate - some shared data",
                    RomSetType::NonMerged => "🔴 Least efficient - duplicated data in clones",
                    RomSetType::Unknown => "❓ Unknown efficiency",
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    app.show_rom_set_info = false;
                }

                if matches!(rom_set_type, RomSetType::Merged) && !app.has_audit_file() {
                    if ui.button("🔍 Run Audit Now").clicked() {
                        app.run_mame_audit();
                        app.show_rom_set_info = false;
                    }
                }
            });
        });
    }
}

fn show_context_menu(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_context_menu {
        egui::Area::new(egui::Id::new("context_menu"))
        .fixed_pos(app.context_menu_position)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label("Launch with:");
                ui.separator();

                if let Some(rom) = &app.context_menu_rom {
                    for (idx, mame) in app.config.mame_executables.iter().enumerate() {
                        if ui.button(&mame.name).clicked() {
                            crate::mame_utils::launch_rom_with_mame(
                                rom,
                                &app.config.rom_dirs,
                                &app.config.extra_rom_dirs,
                                &mame.path,
                                &app.config.graphics_config,
                                &app.config.video_settings  // ADD THIS PARAMETER
                            );
                            app.config.game_preferred_mame.insert(rom.clone(), idx);
                            app.save_config();
                            app.show_context_menu = false;
                        }
                    }

                    ui.separator();

                    if ui.button("Cancel").clicked() {
                        app.show_context_menu = false;
                    }
                }
            });
        });
    }
}
