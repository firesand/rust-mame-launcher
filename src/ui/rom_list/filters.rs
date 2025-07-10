use eframe::egui;
use crate::app::MyApp;
use crate::models::{FilterSettings, StatusFilter};

pub fn show_filters(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Search:");
        let text_changed = ui.add(egui::TextEdit::singleline(&mut app.config.filter_settings.search_text)
            .desired_width(300.0)
            .hint_text("Search by name, description, or ROM filename...")).changed();

        if text_changed {
            app.save_config();
        }

        // Add favorites toggle
        ui.separator();
        if ui.toggle_value(&mut app.config.filter_settings.show_favorites_only, "⭐ Favorites Only")
            .on_hover_text("Show only favorite games")
            .clicked()
        {
            app.save_config();
        }

        if ui.button(if app.config.show_filters { "Hide Filters ▲" } else { "Show Filters ▼" }).clicked() {
            app.config.show_filters = !app.config.show_filters;
            app.save_config();
        }

        if ui.button("Clear Filters").clicked() {
            app.config.filter_settings = FilterSettings::default();
            app.save_config();
        }

        show_graphics_preset_selector(app, ui);
    });

    // Advanced Filters Panel
    if app.config.show_filters {
        show_advanced_filters(app, ui);
    }
}

fn show_graphics_preset_selector(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Graphics:");

        let current_preset = app.config.graphics_config.global_preset.clone();
        egui::ComboBox::new("graphics_preset_combo", "")
            .selected_text(&current_preset)
            .width(150.0)
            .show_ui(ui, |ui| {
                for preset in &app.config.graphics_config.presets {
                    let is_selected = preset.name == current_preset;
                    if ui.selectable_label(is_selected, &preset.name)
                        .on_hover_text(&preset.description)
                        .clicked()
                    {
                        app.config.graphics_config.global_preset = preset.name.clone();
                        app.save_config();
                    }
                }
            });
    });
}

fn show_advanced_filters(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        let mut filter_changed = false;

        // Year filter
        filter_changed |= show_year_filter(app, ui);

        // Manufacturer filter
        filter_changed |= show_manufacturer_filter(app, ui);

        // Status filter
        filter_changed |= show_status_filter(app, ui);

        // Content filters
        filter_changed |= show_content_filters(app, ui);

        // ROM loading mode
        ui.separator();
        show_rom_loading_mode(app, ui);

        if filter_changed {
            app.save_config();
        }
    });
}

fn show_year_filter(app: &mut MyApp, ui: &mut egui::Ui) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Year:");
        changed |= ui.add(egui::TextEdit::singleline(&mut app.config.filter_settings.year_from)
            .desired_width(60.0)
            .hint_text("From")).changed();
        ui.label("-");
        changed |= ui.add(egui::TextEdit::singleline(&mut app.config.filter_settings.year_to)
            .desired_width(60.0)
            .hint_text("To")).changed();
    });
    changed
}

fn show_manufacturer_filter(app: &mut MyApp, ui: &mut egui::Ui) -> bool {
    let mut filter_changed = false;

    ui.horizontal(|ui| {
        ui.separator();
        ui.label("Manufacturer:");

        let selected_text = if app.config.filter_settings.manufacturer.is_empty() {
            "All Manufacturers"
        } else {
            &app.config.filter_settings.manufacturer
        };

        if ui.button(format!("{} ▼", selected_text)).clicked() {
            app.manufacturer_dropdown_open = !app.manufacturer_dropdown_open;
        }

        // Show dropdown
        if app.manufacturer_dropdown_open {
            let dropdown_id = ui.make_persistent_id("manufacturer_dropdown");
            egui::Area::new(dropdown_id)
                .order(egui::Order::Foreground)
                .current_pos(ui.cursor().min + egui::vec2(0.0, 5.0))
                .show(ui.ctx(), |ui| {
                    filter_changed |= show_manufacturer_dropdown_content(app, ui);
                });
        }
    });

    filter_changed
}

fn show_manufacturer_dropdown_content(app: &mut MyApp, ui: &mut egui::Ui) -> bool {
    let mut filter_changed = false;

    egui::Frame::popup(ui.style()).show(ui, |ui| {
        ui.set_max_width(300.0);
        ui.set_max_height(400.0);

        // Search box
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut app.manufacturer_search);
            if ui.small_button("✕").clicked() {
                app.manufacturer_search.clear();
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
            // Show all option first
            if ui.selectable_label(
                app.config.filter_settings.manufacturer.is_empty(),
                "All Manufacturers"
            ).clicked() {
                app.config.filter_settings.manufacturer.clear();
                filter_changed = true;
                app.manufacturer_dropdown_open = false;
            }

            // Filter and show manufacturers
            let search_lower = app.manufacturer_search.to_lowercase();
            for manufacturer in &app.all_manufacturers {
                if search_lower.is_empty() || manufacturer.to_lowercase().contains(&search_lower) {
                    if ui.selectable_label(
                        &app.config.filter_settings.manufacturer == manufacturer,
                        manufacturer
                    ).clicked() {
                        app.config.filter_settings.manufacturer = manufacturer.clone();
                        filter_changed = true;
                        app.manufacturer_dropdown_open = false;
                    }
                }
            }
        });
    });

    filter_changed
}

fn show_status_filter(app: &mut MyApp, ui: &mut egui::Ui) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Status:");
        changed |= ui.radio_value(&mut app.config.filter_settings.status_filter, StatusFilter::All, "All").changed();
        changed |= ui.radio_value(&mut app.config.filter_settings.status_filter, StatusFilter::WorkingOnly, "✅ Working").changed();
        changed |= ui.radio_value(&mut app.config.filter_settings.status_filter, StatusFilter::ImperfectOnly, "⚠️ Imperfect").changed();
        changed |= ui.radio_value(&mut app.config.filter_settings.status_filter, StatusFilter::NotWorkingOnly, "⛔ Not Working").changed();
    });
    changed
}

fn show_content_filters(app: &mut MyApp, ui: &mut egui::Ui) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        changed |= ui.checkbox(&mut app.config.filter_settings.show_clones, "Show all clones")
            .on_hover_text("When unchecked, clones are hidden unless you expand their parent game").changed();
        changed |= ui.checkbox(&mut app.config.filter_settings.hide_non_games, "Hide BIOS/Devices").changed();
    });

    ui.horizontal(|ui| {
        ui.label("Hide content:");
        changed |= ui.checkbox(&mut app.config.filter_settings.hide_mahjong, "Mahjong").changed();
        changed |= ui.checkbox(&mut app.config.filter_settings.hide_adult, "Adult/Nude").changed();
        changed |= ui.checkbox(&mut app.config.filter_settings.hide_casino, "Casino/Cards").changed();
    });

    changed
}

fn show_rom_loading_mode(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("ROM Loading:");

        if ui.checkbox(&mut app.config.assume_merged_sets, "⚡ Fast mode (assume merged sets)")
            .on_hover_text(
                "Enable this if you have merged ROM sets where all clone ROMs are inside parent ZIPs.\n\n\
                ✓ Shows all clones instantly without scanning ZIP contents\n\
                ✓ Much faster ROM loading\n\
                ✗ May show clones that don't actually exist if your set is incomplete\n\n\
                When disabled, the launcher scans inside each ZIP file to verify which clones exist."
            ).clicked() {
            app.save_config();
            app.reload_roms();
        }

        // Show current status with color
        if app.config.assume_merged_sets {
            ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "●");
            ui.label("Fast");
        } else {
            ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "●");
            ui.label("Accurate");
        }
    });
}
