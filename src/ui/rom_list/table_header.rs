use eframe::egui;
use crate::app::MyApp;
use crate::models::{SortColumn, SortDirection};

pub fn show_table_header(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            // Fixed columns
            ui.add_sized([30.0, 24.0], egui::Label::new("")); // Space for expand button
            ui.add_sized([30.0, 24.0], egui::Label::new("")); // Space for star button

            // Icon column header if icons are enabled
            if app.config.show_rom_icons {
                ui.add_sized([app.config.icon_size as f32 + 8.0, 24.0],
                             egui::Label::new(egui::RichText::new("Icon").strong().color(egui::Color32::WHITE)));
            }

            ui.add_sized([40.0, 24.0], egui::Label::new("")); // Space for status icon

            // Sortable columns
            add_sortable_header(app, ui, "Game Title", SortColumn::Title, 450.0);
            add_sortable_header_mono(app, ui, "ROM Name", SortColumn::RomName, 100.0);
            add_sortable_header_mono(app, ui, "Year", SortColumn::Year, 60.0);
            add_sortable_header(app, ui, "Manufacturer", SortColumn::Manufacturer, 200.0);
            add_sortable_header(app, ui, "Status", SortColumn::Status, 80.0);
        });
    });
}

fn add_sortable_header(app: &mut MyApp, ui: &mut egui::Ui, label: &str, column: SortColumn, width: f32) {
    let header_text = format!("{} {}",
        label,
        if app.config.sort_column == column {
            if app.config.sort_direction == SortDirection::Ascending { "▲" } else { "▼" }
        } else { "" }
    );

    let response = ui.add_sized([width, 24.0],
        egui::Button::new(egui::RichText::new(header_text).strong().color(egui::Color32::WHITE))
            .frame(false)
    );

    if response.clicked() {
        handle_header_click(app, column);
    }
}

fn add_sortable_header_mono(app: &mut MyApp, ui: &mut egui::Ui, label: &str, column: SortColumn, width: f32) {
    let header_text = format!("{} {}",
        label,
        if app.config.sort_column == column {
            if app.config.sort_direction == SortDirection::Ascending { "▲" } else { "▼" }
        } else { "" }
    );

    let response = ui.add_sized([width, 24.0],
        egui::Button::new(egui::RichText::new(header_text).strong().color(egui::Color32::WHITE).monospace())
            .frame(false)
    );

    if response.clicked() {
        handle_header_click(app, column);
    }
}

fn handle_header_click(app: &mut MyApp, column: SortColumn) {
    if app.config.sort_column == column {
        app.config.sort_direction = match app.config.sort_direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
    } else {
        app.config.sort_column = column;
        app.config.sort_direction = SortDirection::Ascending;
    }
    app.save_config();
}
