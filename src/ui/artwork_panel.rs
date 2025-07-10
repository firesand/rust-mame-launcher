use eframe::egui;
use crate::app::MyApp;
use crate::models::ArtTab;
use crate::rom_utils::load_art_image;

pub fn show_artwork_panel(app: &mut MyApp, ctx: &egui::Context) {
    egui::SidePanel::right("artwork_panel")
        .min_width(340.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for &tab in &[ArtTab::Snapshot, ArtTab::Cabinet, ArtTab::Title, ArtTab::Artwork, ArtTab::History] {
                    let tab_str = match tab {
                        ArtTab::Snapshot => "Snapshot",
                        ArtTab::Cabinet => "Cabinet",
                        ArtTab::Title => "Title",
                        ArtTab::Artwork => "Artwork",
                        ArtTab::History => "History",
                    };
                    if ui.selectable_label(app.config.art_tab == tab, tab_str).clicked() {
                        app.config.art_tab = tab;
                        app.art_texture = None; // reset texture on tab switch
                        app.save_config();
                    }
                }
            });
            ui.separator();

            if let Some(selected_rom) = &app.config.selected_rom {
                let art_type = match app.config.art_tab {
                    ArtTab::Snapshot => "snap",
                    ArtTab::Cabinet => "cabinets",
                    ArtTab::Title => "titles",
                    ArtTab::Artwork => "artwork",
                    ArtTab::History => "history", // no art for History
                };

                if app.config.art_tab != ArtTab::History {
                    // Load the right image for the active tab
                    if app.art_texture.is_none() {
                        if let Some(img) = load_art_image(selected_rom, &app.config.extra_asset_dirs, art_type) {
                            app.art_texture = Some(ctx.load_texture("artwork", img, egui::TextureOptions::default()));
                        }
                    }
                    if let Some(texture) = &app.art_texture {
                        let available_size = ui.available_size_before_wrap();
                        let max_w = available_size.x.min(320.0);
                        let max_h = available_size.y.min(240.0);
                        let image_size = {
                            let [w, h] = texture.size();
                            let scale = (max_w / w as f32).min(max_h / h as f32).min(1.0);
                            egui::vec2(w as f32 * scale, h as f32 * scale)
                        };
                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::Image::from_texture(texture)
                                    .fit_to_exact_size(image_size)
                            );
                        });
                    } else {
                        ui.label("No image available.");
                    }
                } else {
                    // History: could load text from file if desired
                    ui.label("No history available.");
                }

                ui.separator();
                if let Some(meta) = app.game_metadata.get(selected_rom) {
                    ui.heading(&meta.description);
                    ui.label(format!("Year: {}", meta.year));
                    ui.label(format!("Manufacturer: {}", meta.manufacturer));
                    ui.label(format!("Controls: {}", meta.controls));

                    // Show preferred MAME version if set
                    if let Some(pref_idx) = app.config.game_preferred_mame.get(selected_rom) {
                        if let Some(mame) = app.config.mame_executables.get(*pref_idx) {
                            ui.separator();
                            ui.label(format!("Preferred MAME: {}", mame.name));
                        }
                    }
                }
            } else {
                ui.label("Select a game to see details.");
            }
        });
}
