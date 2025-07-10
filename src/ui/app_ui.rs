use eframe::egui;
use crate::app::MyApp;

pub fn update(app: &mut MyApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Apply the selected theme
    app.config.theme.apply(ctx);

    // NEW: Initialize default icon if not already loaded
    if app.default_icon_texture.is_none() && app.config.show_rom_icons {
        app.init_default_icon(ctx);
    }

    // Show loading overlay if needed
    if app.roms_loading || app.audit_in_progress {
        egui::Area::new(egui::Id::new("loading_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let rect = ctx.available_rect();
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_black_alpha(180),
            );

            egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.add_space(20.0);

                    if app.roms_loading {
                        ui.heading("Loading ROM collection...");
                        if !app.audit_progress.is_empty() {
                            ui.label(&app.audit_progress);
                        }

                        // NEW: Show icon loading status
                        if app.config.show_rom_icons {
                            // Check if icons folder exists in extra_asset_dirs
                            let has_icons = app.config.extra_asset_dirs.iter()
                            .any(|dir| dir.join("icons").exists());

                            if has_icons {
                                ui.add_space(10.0);
                                ui.label("Icons will be loaded after ROMs are listed");
                            }
                        }
                    } else if app.audit_in_progress {
                        ui.heading("Auditing ROMs...");
                        ui.label(&app.audit_progress);

                        // Progress bar
                        let progress = if app.audit_progress.contains("complete") {
                            1.0
                        } else if app.audit_progress.contains("scanning") {
                            0.5
                        } else {
                            0.1
                        };
                        ui.add_space(10.0);
                        ui.add(egui::ProgressBar::new(progress).show_percentage());

                        // Time info
                        if let Some(start_time) = app.audit_start_time {
                            ui.add_space(10.0);
                            let elapsed = start_time.elapsed();
                            ui.label(format!("Time elapsed: {}s", elapsed.as_secs()));

                            // Estimate remaining time
                            if progress > 0.1 && progress < 1.0 {
                                let estimated_total = elapsed.as_secs_f32() / progress;
                                let remaining = (estimated_total - elapsed.as_secs_f32()) as u64;
                                ui.label(format!("Estimated time remaining: {}s", remaining));
                            }
                        }
                    }

                    ui.add_space(10.0);
                    ui.label("Please wait...");
                });
            });
        });

        ctx.request_repaint(); // Keep animating
    }

    // NEW: Show icon loading overlay if many icons are queued
    else if app.icon_load_queue.len() > 50 && app.config.show_rom_icons {
        egui::Area::new(egui::Id::new("icon_loading_overlay"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -40.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Loading icons: {} remaining", app.icon_load_queue.len()));
                });
            });
        });

        ctx.request_repaint(); // Keep updating while loading icons
    }

    // NEW: Show running games overlay if there are active games
    if !app.running_games.is_empty() {
        egui::Area::new(egui::Id::new("running_games_overlay"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -40.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🎮");
                    ui.label(format!("{} game{} running",
                                     app.running_games.len(),
                                     if app.running_games.len() == 1 { "" } else { "s" }
                    ));

                    // Optional: Show game names
                    if ui.small_button("Details").clicked() {
                        // You could add a toggle to show/hide running game details
                    }
                });

                // Optional: List running games on hover
                let response = ui.label("Details");
                response.on_hover_ui(|ui| {
                    ui.label("Running games:");
                    for (rom_name, (_, start_time)) in &app.running_games {
                        let elapsed = start_time.elapsed();
                        let minutes = elapsed.as_secs() / 60;
                        let seconds = elapsed.as_secs() % 60;
                        ui.label(format!("• {} ({}:{:02})", rom_name, minutes, seconds));
                    }
                });
            });
        });
    }

    // Handle close event
    if ctx.input(|i| i.viewport().close_requested()) && !app.show_close_dialog {
        app.show_close_dialog = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
    }

    // Check for background tasks
    check_background_tasks(app, ctx);

    // Show dialogs
    super::dialogs::show_dialogs(app, ctx);

    // Show menu bar
    super::menu_bar::show_menu_bar(app, ctx);

    // Show artwork panel
    super::artwork_panel::show_artwork_panel(app, ctx);

    // Show main panel with ROM list
    super::rom_list::show_rom_list(app, ctx);

    // Show status bar
    super::status_bar::show_status_bar(app, ctx);
}

// NEW: Check for background tasks and update progress
fn check_background_tasks(app: &mut MyApp, ctx: &egui::Context) {
    // Check for ROM loading updates
    if let Some(rx) = &app.audit_tx {
        if let Ok(progress) = rx.try_recv() {
            app.audit_progress = progress.clone();
            app.last_audit_progress = progress.clone();

            // Check for completion
            if progress.contains("AUDIT_COMPLETE") {
                app.audit_in_progress = false;
                app.config.last_audit_time = Some(chrono::Local::now().to_rfc3339());

                // Store audit time for this specific MAME version
                if app.config.selected_mame_index < app.config.mame_executables.len() {
                    if let Some(current_mame) = app.config.mame_executables.get(app.config.selected_mame_index) {
                        let mame_id = app.get_mame_identifier(current_mame);
                        app.config.mame_audit_times.insert(mame_id, chrono::Local::now().to_rfc3339());
                    }
                }

                app.save_config();
                app.reload_roms();
                app.audit_tx = None;
            } else if progress.contains("AUDIT_FAILED") {
                app.audit_in_progress = false;
                app.audit_tx = None;
            }
        }
    }

    // NEW: Check for finished games and update statistics
    app.check_running_games();

    // Request repaint if we have background tasks or running games
    if app.roms_loading || app.audit_in_progress || !app.icon_load_queue.is_empty() || !app.running_games.is_empty() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        update(self, ctx, frame);
    }
}
