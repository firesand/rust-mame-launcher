use eframe::{egui, NativeOptions};
use image::GenericImageView;

mod config;
mod models;
mod ui;
mod rom_utils;
mod mame_utils;
mod app;
mod graphics_presets;  // ← ADD THIS

use app::MyApp;

fn main() -> Result<(), eframe::Error> {
    // Load icon for the application
    let icon_data = if let Ok(image) = image::open("assets/RMAMEUI.png") {
        let (width, height) = image.dimensions();
        let rgba = image.to_rgba8().into_raw();
        Some(egui::IconData {
            rgba,
            width,
            height,
        })
    } else {
        None // If icon loading fails, proceed without icon
    };

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_icon(icon_data.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "RMAMEUI",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
