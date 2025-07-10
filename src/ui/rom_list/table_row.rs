use eframe::egui;
use std::collections::HashMap;
use crate::app::MyApp;
use crate::models::RomStatus;
use crate::mame_utils::launch_rom_with_mame_tracked;
use super::sorting::SortableRom;

pub fn render_rom_row(
    app: &mut MyApp,
    ui: &mut egui::Ui,
    _ctx: &egui::Context,
    rom_data: &SortableRom,
    virtual_parents: &HashMap<String, String>,
    parent_to_clones: &HashMap<String, Vec<(String, String)>>,
                      row: usize,
                      row_height: f32,
) {
    let (display_name, filename, is_clone, has_clones) = rom_data;
    let is_selected = app.config.selected_rom.as_deref() == Some(filename);

    // Draw alternating row background
    draw_row_background(ui, row, row_height);

    // Get metadata
    let metadata_cloned = app.game_metadata.get(filename).cloned();
    let clean_title = get_clean_title(display_name, filename, virtual_parents, &metadata_cloned);
    let year = metadata_cloned.as_ref().map(|m| m.year.clone()).unwrap_or_default();
    let manuf = metadata_cloned.as_ref().map(|m| m.manufacturer.clone()).unwrap_or_default();
    let is_virtual = virtual_parents.contains_key(filename);

    ui.horizontal(|ui| {
        ui.set_min_height(row_height);

        // Expand/collapse button
        render_expand_button(app, ui, filename, is_clone, has_clones, row_height);

        // Favorite star
        render_favorite_button(app, ui, filename, is_virtual, row_height);

        // Icon
        if app.config.show_rom_icons {
            render_rom_icon(app, ui, filename, is_virtual, row_height);
        }

        // Status icon
        render_status_icon(ui, &metadata_cloned, is_virtual, row_height);

        // Title
        let response = render_title(
            ui,
            &clean_title,
            filename,
            is_clone,
            has_clones,
            is_virtual,
            is_selected,
            row_height,
            parent_to_clones
        );

        // Pass response by value, not reference, and remove ctx parameter
        handle_title_interactions(app, response, filename, is_virtual, &metadata_cloned);

        // ROM name
        render_rom_name(ui, filename, is_virtual, row_height);

        // Year
        render_year(ui, &year, row_height);

        // Manufacturer
        render_manufacturer(ui, &manuf, row_height);

        // Status
        render_status(ui, &metadata_cloned, is_virtual, row_height);
    });

    // Draw separator line
    draw_separator_line(ui);
}

fn draw_row_background(ui: &mut egui::Ui, row: usize, row_height: f32) {
    let row_color = if row % 2 == 0 {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0) // Transparent
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 5) // Very subtle white
    };

    if row_color.a() > 0 {
        let row_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(
            egui::Rect::from_min_size(row_rect.min, egui::vec2(row_rect.width(), row_height)),
                                 0.0,
                                 row_color
        );
    }
}

fn get_clean_title(
    display_name: &str,
    filename: &str,
    virtual_parents: &HashMap<String, String>,
    metadata: &Option<crate::models::GameMetadata>
) -> String {
    if virtual_parents.contains_key(filename) {
        display_name.to_string()
    } else {
        metadata.as_ref()
        .map(|m| m.description.clone())
        .unwrap_or_else(|| filename.to_string())
    }
}

fn render_expand_button(
    app: &mut MyApp,
    ui: &mut egui::Ui,
    filename: &str,
    is_clone: &bool,
    has_clones: &bool,
    row_height: f32
) {
    if *is_clone {
        ui.add_sized([30.0, row_height], egui::Label::new("↳"));
    } else if *has_clones {
        let is_expanded = *app.expanded_parents.get(filename).unwrap_or(&false);
        let btn_text = if is_expanded { "▼" } else { "▶" };

        let btn = egui::Button::new(btn_text).min_size(egui::vec2(24.0, 20.0));
        let btn_response = ui.add_sized([30.0, row_height], btn);

        if btn_response.on_hover_text("Click to show/hide clones").clicked() {
            app.expanded_parents.insert(filename.to_string(), !is_expanded);
            println!("Toggled {} to expanded={}", filename, !is_expanded);
        }
    } else {
        ui.add_sized([30.0, row_height], egui::Label::new(""));
    }
}

fn render_favorite_button(
    app: &mut MyApp,
    ui: &mut egui::Ui,
    filename: &str,
    is_virtual: bool,
    row_height: f32
) {
    let is_favorite = app.config.favorite_games.contains(filename);
    if ui.add_sized([30.0, row_height], egui::Button::new(if is_favorite { "⭐" } else { "☆" }))
        .on_hover_text(if is_favorite { "Remove from favorites" } else { "Add to favorites" })
        .clicked() && !is_virtual
        {
            app.toggle_favorite(filename);
        }
}

fn render_rom_icon(
    app: &mut MyApp,
    ui: &mut egui::Ui,
    filename: &str,
    is_virtual: bool,
    row_height: f32
) {
    if is_virtual {
        ui.add_sized([app.config.icon_size as f32 + 8.0, row_height], egui::Label::new(""));
    } else {
        let icon_texture = app.get_rom_icon(filename);
        ui.add_sized(
            [app.config.icon_size as f32 + 8.0, row_height],
            egui::Image::new(&icon_texture)
            .fit_to_exact_size(egui::Vec2::splat(app.config.icon_size as f32))
        );
    }
}

fn render_status_icon(
    ui: &mut egui::Ui,
    metadata: &Option<crate::models::GameMetadata>,
    is_virtual: bool,
    row_height: f32
) {
    let status_icon = if is_virtual {
        "❌"
    } else if let Some(meta) = metadata {
        meta.get_status().to_icon()
    } else {
        "❓"
    };

    ui.add_sized([40.0, row_height], egui::Label::new(status_icon));
}

fn render_title(
    ui: &mut egui::Ui,
    clean_title: &str,
    filename: &str,
    is_clone: &bool,
    has_clones: &bool,
    is_virtual: bool,
    is_selected: bool,
    row_height: f32,
    parent_to_clones: &HashMap<String, Vec<(String, String)>>
) -> egui::Response {
    let title_style = if *is_clone {
        egui::TextStyle::Body
    } else {
        egui::TextStyle::Button
    };

    let title_color = if is_virtual {
        egui::Color32::from_rgb(180, 180, 180)
    } else if *is_clone {
        egui::Color32::from_rgb(200, 200, 200)
    } else {
        egui::Color32::from_rgb(255, 255, 255)
    };

    let formatted_title = if *is_clone {
        format!("    └─ {}", clean_title)
    } else if *has_clones {
        let count = parent_to_clones.get(filename).map(|c| c.len()).unwrap_or(0);
        if count > 0 {
            format!("{} (+{} variants)", clean_title, count)
        } else {
            clean_title.to_string()
        }
    } else {
        clean_title.to_string()
    };

    if is_virtual {
        ui.add_sized([450.0, row_height],
                     egui::Label::new(
                         egui::RichText::new(formatted_title)
                         .color(title_color)
                         .text_style(title_style)
                     )
                     .sense(egui::Sense::hover())
        )
    } else {
        ui.add_sized([450.0, row_height],
                     egui::SelectableLabel::new(
                         is_selected,
                         egui::RichText::new(formatted_title)
                         .color(title_color)
                         .text_style(title_style)
                     )
        )
    }
}

fn handle_title_interactions(
    app: &mut MyApp,
    response: egui::Response,  // Take ownership instead of reference
    filename: &str,
    is_virtual: bool,
    metadata: &Option<crate::models::GameMetadata>,
) {
    if response.clicked() && !is_virtual {
        app.config.selected_rom = Some(filename.to_string());
        app.art_texture = None;
        app.save_config();
    }

    if response.double_clicked() && !is_virtual && !app.config.mame_executables.is_empty() {
        let mame_idx = app.config.game_preferred_mame.get(filename)
        .copied()
        .unwrap_or(app.config.selected_mame_index);
        if let Some(mame) = app.config.mame_executables.get(mame_idx) {
            match launch_rom_with_mame_tracked(
                filename,
                &app.config.rom_dirs,
                &app.config.extra_rom_dirs,
                &mame.path,
                &app.config.graphics_config,
                &app.config.video_settings  // ADD THIS LINE
            ) {
                Ok(child) => {
                    app.running_games.insert(filename.to_string(), (child, std::time::Instant::now()));
                    println!("Started tracking game: {}", filename);
                }
                Err(e) => {
                    println!("Failed to launch game: {}", e);
                }
            }
        }
    }

    // Clone metadata for the closures
    let metadata_for_hover = metadata.clone();
    let stats_for_hover = app.config.game_stats.get(filename).cloned();
    let filename_owned = filename.to_string();

    // Add hover tooltip and context menu
    let response = response
    .on_hover_ui_at_pointer(|ui| {
        show_rom_tooltip(ui, &metadata_for_hover, &stats_for_hover);
    });

    // Context menu (only if not virtual)
    if !is_virtual {
        response.context_menu(|ui| {
            show_rom_context_menu(app, ui, &filename_owned);
        });
    }
}

fn show_rom_tooltip(
    ui: &mut egui::Ui,
    metadata: &Option<crate::models::GameMetadata>,
    stats: &Option<crate::models::GameStats>
) {
    if let Some(meta) = metadata {
        ui.strong(&meta.description);
        ui.label(format!("Year: {}", meta.year));
        ui.label(format!("Manufacturer: {}", meta.manufacturer));
        if !meta.controls.is_empty() {
            ui.label(format!("Controls: {}", meta.controls));
        }
        if meta.is_clone {
            if let Some(parent) = &meta.parent {
                ui.label(format!("Clone of: {}", parent));
            }
        }
        let status = meta.get_status();
        ui.colored_label(status.to_color(), format!("Status: {:?}", status));

        if let Some(stats) = stats {
            ui.separator();
            ui.label(format!("Times Played: {}", stats.play_count));
            if let Some(last_played) = &stats.last_played {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_played) {
                    let local: chrono::DateTime<chrono::Local> = dt.into();
                    ui.label(format!("Last Played: {}", local.format("%Y-%m-%d %H:%M")));
                }
            }
            let hours = stats.total_play_time / 3600;
            let minutes = (stats.total_play_time % 3600) / 60;
            ui.label(format!("Total Play Time: {}h {}m", hours, minutes));
        }
    }
}

fn show_rom_context_menu(app: &mut MyApp, ui: &mut egui::Ui, filename: &str) {
    let is_favorite = app.config.favorite_games.contains(filename);

    if ui.button(if is_favorite { "Remove from Favorites ⭐" } else { "Add to Favorites ☆" }).clicked() {
        app.toggle_favorite(filename);
        ui.close_menu();
    }

    ui.separator();

    ui.label("Launch with:");
    ui.separator();
    for (idx, mame) in app.config.mame_executables.iter().enumerate() {
        if ui.button(&mame.name).clicked() {
            match launch_rom_with_mame_tracked(
                filename,
                &app.config.rom_dirs,
                &app.config.extra_rom_dirs,
                &mame.path,
                &app.config.graphics_config,
                &app.config.video_settings  // ADD THIS LINE
            ) {
                Ok(child) => {
                    app.running_games.insert(filename.to_string(), (child, std::time::Instant::now()));
                    app.config.game_preferred_mame.insert(filename.to_string(), idx);
                    app.save_config();
                    println!("Started tracking game: {}", filename);
                }
                Err(e) => {
                    println!("Failed to launch game: {}", e);
                }
            }
            ui.close_menu();
        }
    }
}

fn render_rom_name(ui: &mut egui::Ui, filename: &str, is_virtual: bool, row_height: f32) {
    let rom_text = if is_virtual {
        "(missing)".to_string()
    } else {
        let max_width = 12;
        if filename.len() > max_width {
            format!("{}…", &filename[..max_width-1])
        } else {
            format!("{:<width$}", filename, width = max_width)
        }
    };

    ui.add_sized([100.0, row_height], egui::Label::new(
        egui::RichText::new(rom_text)
        .color(egui::Color32::from_rgb(200, 200, 200))
        .monospace()
    ));
}

fn render_year(ui: &mut egui::Ui, year: &str, row_height: f32) {
    let year_text = if year.is_empty() {
        "----".to_string()
    } else {
        format!("{:>4}", year)
    };

    ui.add_sized([60.0, row_height], egui::Label::new(
        egui::RichText::new(year_text)
        .color(egui::Color32::from_rgb(200, 200, 200))
        .monospace()
    ));
}

fn render_manufacturer(ui: &mut egui::Ui, manuf: &str, row_height: f32) {
    let manuf_text = if manuf.is_empty() {
        "Unknown".to_string()
    } else {
        let max_width = 25;
        if manuf.len() > max_width {
            format!("{}…", &manuf[..max_width-1])
        } else {
            manuf.to_string()  // Convert to String
        }
    };

    ui.add_sized([200.0, row_height], egui::Label::new(
        egui::RichText::new(manuf_text).color(egui::Color32::from_rgb(200, 200, 200))
    ));
}

fn render_status(
    ui: &mut egui::Ui,
    metadata: &Option<crate::models::GameMetadata>,
    is_virtual: bool,
    row_height: f32
) {
    let (status_text, status_color) = if is_virtual {
        ("missing", egui::Color32::from_rgb(128, 128, 128))
    } else if let Some(meta) = metadata {
        let status = meta.get_status();
        (
            match status {
                RomStatus::Good => "good",
                RomStatus::Imperfect => "imperfect",
                RomStatus::Preliminary => "prelim",
                RomStatus::NotWorking => "not work",
            },
         status.to_color()
        )
    } else {
        ("unknown", egui::Color32::from_rgb(128, 128, 128))
    };

    ui.add_sized([80.0, row_height],
                 egui::Label::new(
                     egui::RichText::new(format!("{:<9}", status_text))
                     .color(status_color)
                     .size(11.0)
                     .monospace()
                 )
    );
}

fn draw_separator_line(ui: &mut egui::Ui) {
    let sep_rect = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [sep_rect.left_bottom(), sep_rect.right_bottom()],
                              (0.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20))
    );
}
