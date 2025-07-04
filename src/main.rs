use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;
use eframe::{egui, NativeOptions};
use image::GenericImageView; // Ensure you have this for image loading
use eframe::egui::ColorImage;
use eframe::egui::TextureOptions; // Import TextureOptions
use std::thread;
use std::sync::mpsc;
use rayon::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct GameMetadata {
    pub name: String,
    pub description: String,
    pub year: String,
    pub manufacturer: String,
    pub controls: String,
    pub is_device: bool,
    pub is_bios: bool,
    pub is_mechanical: bool,
    pub runnable: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ArtTab {
    Snapshot,
    Cabinet,
    Title,
    Artwork,
    History,
}

impl Default for ArtTab {
    fn default() -> Self { ArtTab::Snapshot }
}

#[derive(Clone, Debug, Default)]
pub struct FilterSettings {
    pub search_text: String,
    pub year_from: String,
    pub year_to: String,
    pub manufacturer: String,
    pub selected_manufacturers: Vec<String>,
    pub show_clones: bool,
    pub show_working_only: bool,
    pub hide_non_games: bool,
    pub hide_mahjong: bool,
    pub hide_adult: bool,
    pub hide_casino: bool,
}

pub struct MyApp {
    pub mame_executable: String,
    pub rom_dirs: Vec<PathBuf>,
    pub extra_rom_dirs: Vec<PathBuf>,
    pub extra_asset_dirs: Vec<PathBuf>,
    pub mame_titles: HashMap<String, String>,
    pub roms: Vec<(String, String)>,
    pub roms_loading: bool,
    pub roms_tx: Option<mpsc::Receiver<Vec<(String, String)>>>,
    pub selected_rom: Option<String>,
    pub screenshot: Option<ColorImage>,
    pub texture_handle: Option<egui::TextureHandle>,
    pub game_metadata: HashMap<String, GameMetadata>,
    pub art_tab: ArtTab,
    pub art_texture: Option<egui::TextureHandle>,
    pub filter_settings: FilterSettings,
    pub all_manufacturers: Vec<String>,
    pub show_filters: bool,
    pub show_about: bool,
    pub total_games_count: usize,
    pub working_games_count: usize,
    pub mame_version: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            rom_dirs: vec![],
            extra_rom_dirs: vec![],
            extra_asset_dirs: vec![],
            mame_titles: HashMap::new(),
            roms: vec![],
            roms_loading: false,
            roms_tx: None,
            selected_rom: None,
            screenshot: None,
            texture_handle: None,
            mame_executable: "mame".to_string(),
            game_metadata: HashMap::new(),
            art_tab: ArtTab::Snapshot,
            art_texture: None,
            filter_settings: FilterSettings {
                hide_non_games: true,  // Default to hiding non-games
                hide_adult: true,      // Default to hiding adult content
                ..Default::default()
            },
            all_manufacturers: Vec::new(),
            show_filters: false,
            show_about: false,
            total_games_count: 0,
            working_games_count: 0,
            mame_version: String::new(),
        }
    }
}

fn get_mame_version(exec_path: &str) -> String {
    if let Ok(output) = Command::new(exec_path)
        .arg("-version")
        .output() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            version_str.lines().next().unwrap_or("Unknown").to_string()
        } else {
            "Unknown".to_string()
        }
}

fn load_mame_metadata_parallel_with_exec(exec_path: &str) -> HashMap<String, GameMetadata> {
    let output = Command::new(exec_path)
    .arg("-listxml")
    .output()
    .expect("Failed to run mame -listxml");

    let xml_str = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<_> = xml_str.split("<machine ").skip(1).collect();

    entries
    .into_par_iter()
    .filter_map(|entry| {
        let name = entry.lines().find(|l| l.contains("name=")).and_then(|l| l.split('"').nth(1))?;

        // Check if it's a device, BIOS, or mechanical machine
        let is_device = entry.contains("isdevice=\"yes\"");
        let is_bios = entry.contains("isbios=\"yes\"");
        let is_mechanical = entry.contains("ismechanical=\"yes\"");
        let runnable = entry.contains("runnable=\"no\"");

        let description = entry.lines()
        .find(|l| l.contains("<description>"))
        .and_then(|l| l.split_once('>'))
        .and_then(|(_, r)| r.split_once('<'))
        .map(|(d, _)| d.trim().to_string())
        .unwrap_or_default();
        let year = entry.lines()
        .find(|l| l.contains("<year>"))
        .and_then(|l| l.split_once('>'))
        .and_then(|(_, r)| r.split_once('<'))
        .map(|(d, _)| d.trim().to_string())
        .unwrap_or_default();
        let manufacturer = entry.lines()
        .find(|l| l.contains("<manufacturer>"))
        .and_then(|l| l.split_once('>'))
        .and_then(|(_, r)| r.split_once('<'))
        .map(|(d, _)| d.trim().to_string())
        .unwrap_or_default();
        let controls = entry.lines()
        .find(|l| l.contains("<control "))
        .and_then(|l| l.split("type=\"").nth(1))
        .and_then(|s| s.split('"').next())
        .unwrap_or_default()
        .to_string();

        Some((
            name.to_string(),
              GameMetadata {
                  name: name.to_string(),
              description,
              year,
              manufacturer,
              controls,
              is_device,
              is_bios,
              is_mechanical,
              runnable,
              },
        ))
    })
    .collect()
}

fn collect_roms_from_dirs(dirs: &[PathBuf], _exec_path: &str, mame_titles: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut roms = Vec::new();
    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        let title = mame_titles.get(name).cloned().unwrap_or_else(|| name.to_uppercase());
                        let display_name = format!("{} [{}]", title, name);
                        roms.push((display_name, name.to_string()));
                    }
                }
            }
        }
    }
    roms.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    roms
}

// Load an image of a given type for a rom, e.g. snap/cabinets/titles/artwork
fn load_art_image(rom: &str, asset_dirs: &[PathBuf], art_type: &str) -> Option<ColorImage> {
    let extensions = ["png", "jpg"];
    for dir in asset_dirs {
        let art_dir = dir.join(art_type);  // e.g., "snap", "cabinet", "titles", "artwork"

        // Ensure that the folder exists
        if !art_dir.exists() {
            println!("Folder not found: {}", art_dir.display());
            continue;
        }

        // Special handling for artwork folder with ZIP files
        if art_type == "artwork" {
            let zip_path = art_dir.join(format!("{}.zip", rom));
            if zip_path.exists() {
                println!("Found artwork ZIP: {}", zip_path.display());
                if let Ok(file) = std::fs::File::open(&zip_path) {
                    if let Ok(mut archive) = zip::ZipArchive::new(file) {
                        // Collect all image files in the ZIP
                        let mut image_files: Vec<(String, usize)> = Vec::new();

                        for i in 0..archive.len() {
                            if let Ok(file) = archive.by_index(i) {
                                let name = file.name().to_lowercase();
                                if (name.ends_with(".png") || name.ends_with(".jpg")) && !name.contains('/') {
                                    image_files.push((file.name().to_string(), i));
                                }
                            }
                        }

                        // Sort by priority: look for marquee, bezel, then instruction cards, then any image
                        image_files.sort_by_key(|(name, _)| {
                            let lower = name.to_lowercase();
                            if lower.contains("marquee") { 0 }
                            else if lower.contains("bezel") { 1 }
                            else if lower.contains("inst") { 2 }
                            else if lower.contains("cpo") || lower.contains("control") { 3 }
                            else { 4 }
                        });

                        // Try to load the first suitable image
                        for (name, index) in image_files {
                            if let Ok(mut file) = archive.by_index(index) {
                                let mut buffer = Vec::new();
                                if std::io::Read::read_to_end(&mut file, &mut buffer).is_ok() {
                                    if let Ok(decoded) = image::load_from_memory(&buffer) {
                                        println!("Loading artwork image: {}", name);
                                        let rgba = decoded.to_rgba8();
                                        let size = [rgba.width() as usize, rgba.height() as usize];
                                        let pixels = rgba.into_vec();
                                        return Some(ColorImage::from_rgba_unmultiplied(size, &pixels));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Standard image loading for other folders
        for ext in &extensions {
            let path = art_dir.join(format!("{}.{}", rom, ext));
            if path.exists() {
                println!("Loading art from: {}", path.display());  // Debug print
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(decoded) = image::load_from_memory(&bytes) {
                        let rgba = decoded.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_vec();
                        return Some(ColorImage::from_rgba_unmultiplied(size, &pixels));
                    } else {
                        println!("Failed to decode image: {}", path.display());
                    }
                } else {
                    println!("Failed to read image file: {}", path.display());
                }
            }
        }
    }
    println!("No image found for {} in {}", rom, art_type); // Debug print
    None
}

fn launch_rom(rom: &str, rom_dirs: &[PathBuf], extra_dirs: &[PathBuf], mame_executable: &str) {
    println!("Launching ROM: {}", rom);
    let all_dirs = rom_dirs.iter().chain(extra_dirs.iter());
    let separator = ";";
    let rom_paths = all_dirs
    .map(|p| p.to_string_lossy())
    .collect::<Vec<_>>()
    .join(separator);
    let _ = Command::new(mame_executable)
    .arg(rom)
    .arg("-rompath")
    .arg(rom_paths)
    .spawn();
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // ---- Menu Bar ----
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File Menu
                ui.menu_button("File", |ui| {
                    if ui.button("Set MAME Executable...").clicked() {
                        let file_dialog = if cfg!(target_os = "linux") {
                            // On Linux, don't filter by extension
                            rfd::FileDialog::new()
                            .set_file_name("mame")
                        } else if cfg!(target_os = "windows") {
                            // On Windows, filter for .exe files
                            rfd::FileDialog::new()
                            .add_filter("Executable", &["exe"])
                        } else {
                            // On other platforms, allow any file
                            rfd::FileDialog::new()
                        };

                        if let Some(path) = file_dialog.pick_file() {
                            if let Some(path_str) = path.to_str() {
                                self.mame_executable = path_str.to_string();
                                // Reload metadata with new executable
                                if !self.rom_dirs.is_empty() {
                                    self.mame_version = get_mame_version(&self.mame_executable);
                                    self.game_metadata = load_mame_metadata_parallel_with_exec(&self.mame_executable);
                                    self.mame_titles = self.game_metadata.iter().map(|(k, v)| (k.clone(), v.description.clone())).collect();

                                    // Count total and working games
                                    self.total_games_count = self.game_metadata.iter()
                                    .filter(|(_, meta)| !meta.is_device && !meta.is_bios)
                                    .count();
                                    self.working_games_count = self.game_metadata.iter()
                                    .filter(|(_, meta)| !meta.is_device && !meta.is_bios && !meta.is_mechanical && !meta.runnable)
                                    .count();

                                    // Extract unique manufacturers
                                    let mut manufacturers: Vec<String> = self.game_metadata.values()
                                    .map(|m| m.manufacturer.clone())
                                    .filter(|m| !m.is_empty())
                                    .collect();
                                    manufacturers.sort();
                                    manufacturers.dedup();
                                    self.all_manufacturers = manufacturers;
                                }
                            }
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Options Menu
                ui.menu_button("Options", |ui| {
                    ui.menu_button("Directories", |ui| {
                        if ui.button("Add ROM Path...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.rom_dirs.push(path.clone());
                                self.extra_rom_dirs.push(path); // Same path for ROMs, CHDs, and BIOS
                                self.mame_version = get_mame_version(&self.mame_executable);
                                self.game_metadata = load_mame_metadata_parallel_with_exec(&self.mame_executable);
                                self.mame_titles = self.game_metadata.iter().map(|(k, v)| (k.clone(), v.description.clone())).collect();

                                // Count total and working games
                                self.total_games_count = self.game_metadata.iter()
                                .filter(|(_, meta)| !meta.is_device && !meta.is_bios)
                                .count();
                                self.working_games_count = self.game_metadata.iter()
                                .filter(|(_, meta)| !meta.is_device && !meta.is_bios && !meta.is_mechanical && !meta.runnable)
                                .count();

                                // Extract unique manufacturers
                                let mut manufacturers: Vec<String> = self.game_metadata.values()
                                .map(|m| m.manufacturer.clone())
                                .filter(|m| !m.is_empty())
                                .collect();
                                manufacturers.sort();
                                manufacturers.dedup();
                                self.all_manufacturers = manufacturers;

                                let rom_dirs = self.rom_dirs.clone();
                                let mame_titles = self.mame_titles.clone();
                                let mame_executable = self.mame_executable.clone();
                                let (tx, rx) = mpsc::channel();
                                self.roms_tx = Some(rx);
                                self.roms_loading = true;
                                thread::spawn(move || {
                                    let roms = collect_roms_from_dirs(&rom_dirs, &mame_executable, &mame_titles);
                                    let _ = tx.send(roms);
                                });
                            }
                            ui.close_menu();
                        }

                        if ui.button("Add Extras Path...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.extra_asset_dirs.push(path);
                            }
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button("Clear All Paths").clicked() {
                            self.rom_dirs.clear();
                            self.extra_rom_dirs.clear();
                            self.extra_asset_dirs.clear();
                            self.roms.clear();
                            self.selected_rom = None;
                            self.screenshot = None;
                            self.filter_settings = FilterSettings::default();
                            self.all_manufacturers.clear();
                            self.art_texture = None;
                            ui.close_menu();
                        }
                    });
                });

                // Help Menu
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // ---- About Dialog ----
        if self.show_about {
            egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Rust MAME Launcher");
                    ui.add_space(10.0);
                    ui.label("Version 0.1.0");
                    ui.add_space(10.0);
                    ui.label("A MAME frontend built with Rust");
                    ui.add_space(5.0);
                    ui.label("Created by Edo Hikmahtiar - Indonesia");
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label("Features:");
                    ui.label("• Multi-folder ROM management");
                    ui.label("• Game artwork display");
                    ui.label("• Advanced search and filtering");
                    ui.label("• MAME metadata integration");
                    ui.add_space(20.0);
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
            });
        }

        // ---- Right-side Artwork Panel ----
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
                    if ui.selectable_label(self.art_tab == tab, tab_str).clicked() {
                        self.art_tab = tab;
                        self.art_texture = None; // reset texture on tab switch
                    }
                }
            });
            ui.separator();

            if let Some(selected_rom) = &self.selected_rom {
                let art_type = match self.art_tab {
                    ArtTab::Snapshot => "snap",
                    ArtTab::Cabinet => "cabinets",
                    ArtTab::Title => "titles",
                    ArtTab::Artwork => "artwork",
                    ArtTab::History => "history", // no art for History
                };

                if self.art_tab != ArtTab::History {
                    // Load the right image for the active tab
                    if self.art_texture.is_none() {
                        if let Some(img) = load_art_image(selected_rom, &self.extra_asset_dirs, art_type) {
                            self.art_texture = Some(ctx.load_texture("artwork", img, TextureOptions::default()));
                        }
                    }
                    if let Some(texture) = &self.art_texture {
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
                if let Some(meta) = self.game_metadata.get(selected_rom) {
                    ui.heading(&meta.description);
                    ui.label(format!("Year: {}", meta.year));
                    ui.label(format!("Manufacturer: {}", meta.manufacturer));
                    ui.label(format!("Controls: {}", meta.controls));
                }
            } else {
                ui.label("Select a game to see details.");
            }
        });

        // ---- Main Panel with ROM List ----
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust MAME Launcher");

            // Show current MAME executable
            ui.horizontal(|ui| {
                ui.label("MAME Executable:");
                ui.label(&self.mame_executable);
                ui.small_button("📁").on_hover_text("Change in File menu");
            });

            if !self.rom_dirs.is_empty() || !self.extra_asset_dirs.is_empty() {
                ui.group(|ui| {
                    ui.label("Configured Paths:");

                    // Show ROM paths (which include ROMs, CHDs, and BIOS)
                    if !self.rom_dirs.is_empty() {
                        ui.label("ROM Paths (ROMs/CHDs/BIOS):");
                        for (i, dir) in self.rom_dirs.iter().enumerate() {
                            ui.label(format!("  {}. {}", i + 1, dir.display()));
                        }
                    }

                    // Show Extras paths
                    if !self.extra_asset_dirs.is_empty() {
                        ui.label("Extras Paths (Artwork/Snapshots):");
                        for (i, dir) in self.extra_asset_dirs.iter().enumerate() {
                            ui.label(format!("  {}. {}", i + 1, dir.display()));
                        }
                    }
                });
            } else {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100),
                                 "No paths configured. Use Options → Directories to add paths.");
            }

            ui.separator();

            // Search and Filter Section
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(egui::TextEdit::singleline(&mut self.filter_settings.search_text)
                .desired_width(300.0)
                .hint_text("Search by name, description, or ROM filename..."));

                if ui.button(if self.show_filters { "Hide Filters ▲" } else { "Show Filters ▼" }).clicked() {
                    self.show_filters = !self.show_filters;
                }

                if ui.button("Clear Filters").clicked() {
                    self.filter_settings = FilterSettings::default();
                }
            });

            // Advanced Filters Panel
            if self.show_filters {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Year:");
                        ui.add(egui::TextEdit::singleline(&mut self.filter_settings.year_from)
                        .desired_width(60.0)
                        .hint_text("From"));
                        ui.label("-");
                        ui.add(egui::TextEdit::singleline(&mut self.filter_settings.year_to)
                        .desired_width(60.0)
                        .hint_text("To"));

                        ui.separator();

                        ui.label("Manufacturer:");
                        egui::ComboBox::from_label("")
                        .selected_text(if self.filter_settings.manufacturer.is_empty() {
                            "All Manufacturers"
                        } else {
                            &self.filter_settings.manufacturer
                        })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(self.filter_settings.manufacturer.is_empty(), "All Manufacturers").clicked() {
                                self.filter_settings.manufacturer.clear();
                            }
                            for manufacturer in &self.all_manufacturers {
                                if ui.selectable_label(&self.filter_settings.manufacturer == manufacturer, manufacturer).clicked() {
                                    self.filter_settings.manufacturer = manufacturer.clone();
                                }
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.filter_settings.show_working_only, "Working games only");
                        ui.checkbox(&mut self.filter_settings.show_clones, "Show clones");
                        ui.checkbox(&mut self.filter_settings.hide_non_games, "Hide BIOS/Devices");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Hide content:");
                        ui.checkbox(&mut self.filter_settings.hide_mahjong, "Mahjong");
                        ui.checkbox(&mut self.filter_settings.hide_adult, "Adult/Nude");
                        ui.checkbox(&mut self.filter_settings.hide_casino, "Casino/Cards");
                    });
                });
            }

            ui.separator();

            // Table Header
            ui.horizontal(|ui| {
                ui.add_sized([400.0, 24.0], egui::Label::new(egui::RichText::new("Game Title").strong()));
                ui.add_sized([100.0, 24.0], egui::Label::new(egui::RichText::new("ROM Name").strong()));
                ui.add_sized([60.0, 24.0], egui::Label::new(egui::RichText::new("Year").strong()));
                ui.add_sized([200.0, 24.0], egui::Label::new(egui::RichText::new("Manufacturer").strong()));
            });

            ui.separator();

            let filtered_roms: Vec<(String, String)> = self.roms.iter()
            .filter(|(display, filename)| {
                // Get metadata for filtering
                if let Some(metadata) = self.game_metadata.get(filename) {
                    let desc_lower = metadata.description.to_lowercase();
                    let name_lower = filename.to_lowercase();

                    // Content filtering
                    if self.filter_settings.hide_mahjong {
                        if desc_lower.contains("mahjong") || desc_lower.contains("mah-jong") ||
                            desc_lower.contains("mahjongg") || name_lower.contains("mahjong") ||
                            name_lower.starts_with("mj") || name_lower.starts_with("maj") {
                                return false;
                            }
                    }

                    if self.filter_settings.hide_adult {
                        let adult_keywords = [
                            "nude", "naked", "strip", "adult", "sexy", "erotic",
                            "porn", "xxx", "sex", "topless", "bottomless", "peep",
                            "bikini", "lingerie", "bondage", "hentai", "yakyuken",
                            "suck", "gal", "mistress", "bunny", "night life",
                            "call girl", "hostess", "escort", "pleasure"
                        ];

                        for keyword in &adult_keywords {
                            if desc_lower.contains(keyword) || name_lower.contains(keyword) {
                                // Allow some exceptions for legitimate games
                                if !desc_lower.contains("galaxy") && // Galaxy games
                                    !desc_lower.contains("galaga") && // Galaga variants
                                    !desc_lower.contains("strikers") && // Sports games
                                    !desc_lower.contains("strike") && // Strike games
                                    !desc_lower.contains("poker night") { // Poker Night games
                                        return false;
                                    }
                            }
                        }
                    }

                    if self.filter_settings.hide_casino {
                        let casino_keywords = [
                            "poker", "blackjack", "slot", "casino", "gambling",
                            "roulette", "baccarat", "craps", "keno", "bingo",
                            "lottery", "scratch", "fruit machine", "pachislot",
                            "pachinko", "hanafuda", "cards", "solitaire", "twentyone",
                            "twenty-one", "21", "hold'em", "holdem", "texas", "betting",
                            "wager", "jackpot", "coins", "chips", "dealer"
                        ];

                        for keyword in &casino_keywords {
                            if desc_lower.contains(keyword) || name_lower.contains(keyword) {
                                // Allow some exceptions
                                if !desc_lower.contains("battle") && // Card battle games
                                    !desc_lower.contains("adventure") && // Adventure games
                                    !desc_lower.contains("racing") && // Racing games
                                    !desc_lower.contains("puzzle") && // Puzzle games
                                    !desc_lower.contains("jackpot journey") && // Specific game
                                    !name_lower.contains("jack") { // Games with "jack" in name like "Jumping Jack"
                                        return false;
                                    }
                            }
                        }

                        // Also filter obvious gambling game patterns
                        if name_lower.starts_with("bfm") || // Fruit machines
                            name_lower.starts_with("mpu") || // Fruit machines
                            name_lower.starts_with("sc") && name_lower.len() <= 6 { // Slot codes
                                return false;
                            }
                    }
                }

                // Check if we should filter out non-games
                if self.filter_settings.hide_non_games {
                    if let Some(metadata) = self.game_metadata.get(filename) {
                        // Filter out devices, BIOS, mechanical games, and non-runnable entries
                        if metadata.is_device || metadata.is_bios || metadata.is_mechanical || metadata.runnable {
                            return false;
                        }

                        // Also filter out common non-game patterns in descriptions
                        let desc_lower = metadata.description.to_lowercase();
                        let name_lower = filename.to_lowercase();

                        // Extensive list of non-game keywords
                        let non_game_keywords = [
                            "bios", "system", "monitor", "graphics", "computer", "spectrum",
                            "interface", "terminal", "keyboard", "printer", "plotter",
                            "disk drive", "floppy", "hard disk", "tape drive", "cassette",
                            "development", "debugger", "diagnostic", "test", "demo", "sampler",
                            "workstation", "mainframe", "minicomputer", "microcomputer",
                            "calculator", "organizer", "typewriter", "word processor",
                            "modem", "network", "ethernet", "serial", "parallel",
                            "adapter", "converter", "expansion", "cartridge slot",
                            "cpu", "processor", "memory", "ram", "rom", "eprom",
                            "oscilloscope", "multimeter", "logic analyzer",
                            "synthesizer", "sampler", "drum machine", "sequencer",
                            "mixer", "amplifier", "receiver", "tuner",
                            "vcr", "video", "laserdisc", "dvd", "bluray",
                            "camera", "camcorder", "projector", "display",
                            "phone", "telephone", "answering machine", "fax",
                            "pda", "pocket pc", "palm", "newton",
                            "gps", "navigation", "watch", "clock",
                            "training", "educational", "learning", "tutorial",
                            "service", "diagnostic", "calibration", "alignment",
                            "emulator", "simulator", "development kit", "dev kit",
                            "prototype", "sample", "example", "reference"
                        ];

                        // Check description for non-game keywords
                        for keyword in &non_game_keywords {
                            if desc_lower.contains(keyword) &&
                                !desc_lower.contains("game") &&
                                !desc_lower.contains("arcade") &&
                                !desc_lower.contains("pinball") {
                                    return false;
                                }
                        }

                        // Filter out specific computer/terminal patterns in names
                        let computer_patterns = [
                            "cpc-", "zt-", "zx-", "z-1", "yis", "spc-", "fspc",
                            "rc7", "rc8", "xd8", "wy-", "xbox", "apple", "mac",
                            "amiga", "atari st", "commodore", "pet ", "vic-",
                            "trs-", "coco", "dragon", "oric", "amstrad", "sinclair",
                            "msx", "coleco", "intellivision", "ti-99", "tandy",
                            "kaypro", "osborne", "compaq", "ibm", "pc-", "pc1",
                            "pc2", "pc3", "pc4", "pc5", "pc6", "pc7", "pc8", "pc9",
                            "xt", "at286", "at386", "at486", "pentium",
                            "hp", "dec", "vax", "pdp", "sun", "sgi", "next",
                            "lisa", "newton", "palm", "psion", "sharp", "casio",
                            "epson", "brother", "canon", "lexmark", "okidata",
                            "citizen", "star", "panasonic", "toshiba", "nec",
                            "fujitsu", "hitachi", "mitsubishi", "sony", "sanyo",
                            "jvc", "aiwa", "kenwood", "pioneer", "marantz",
                            "denon", "onkyo", "yamaha", "roland", "korg", "akai",
                            "ensoniq", "emu", "kurzweil", "oberheim", "moog",
                            "sequential", "dave smith", "novation", "arturia",
                            "behringer", "zoom", "boss", "digitech", "line6",
                            "vox", "marshall", "fender", "mesa", "orange",
                            "peavey", "laney", "blackstar", "hughes", "kettner",
                            // New patterns based on your list
                            "rw10", "rw12", "rw24", "digilog", "so30", "3do",
                            "tek41", "tek61", "sis85", "ch4", "att6", "d68",
                            "pegasus", "abpi", "abpm", "abpv", "abc8", "abc80",
                            "dmax", "datacast", "datum", "gl20", "gl30", "gl40",
                            "gl50", "gl60", "glpn", "intlc", "ip20", "ip24",
                            "ip67", "ipb", "ipds", "iq15", "isbc", "itt",
                            "mekd", "n64", "noki", "nws3", "nws8", "p500",
                            "p8000", "pb10", "pb20", "pcm", "phc2", "phc25",
                            "rc20", "rc30", "rc32", "rm38", "rm380",
                            // Additional computer/device patterns
                            "vt10", "vt22", "vt24", "vt32", "vt50", "vt52",
                            "vt10", "vt220", "vt240", "vt320", "vt420",
                            "wyse", "wy-", "tek", "tektronix", "hazeltine",
                            "adds", "adm", "televideo", "tvi", "qume", "zentec",
                            "datapoint", "intecolor", "falco", "esprit",
                            "visual", "freedom", "soroc", "microterm", "mime",
                            "datamedia", "heath", "zenith", "perkin", "elmer",
                            "beehive", "cybernex", "dasher", "delta", "omron"
                        ];

                        // Check if name starts with these patterns
                        for pattern in &computer_patterns {
                            if name_lower.starts_with(pattern) ||
                                (desc_lower.contains(pattern) &&
                                !desc_lower.contains("game") &&
                                !desc_lower.contains("arcade")) {
                                    return false;
                                }
                        }

                        // Additional specific filtering for entries that look like model numbers
                        // Filter entries that are mostly alphanumeric codes
                        // But EXCLUDE legitimate games with numbers
                        let legitimate_game_patterns = [
                            "19", // 1941, 1942, 1943, 1944, 19xx series
                            "20", // 2020bb, etc
                            "kof", // King of Fighters series
                            "sf", // Street Fighter
                            "mk", // Mortal Kombat
                            "mvs", // Neo Geo MVS games
                            "aof", // Art of Fighting
                            "svc", // SNK vs Capcom
                            "rbff", // Real Bout Fatal Fury
                            "samsho", // Samurai Shodown
                            "metal", // Metal Slug
                            "puzzle", // Puzzle games
                            "tetris", // Tetris variants
                            "pac", // Pac-Man variants
                            "mario", // Mario games
                            "sonic", // Sonic games
                            "street", // Street Fighter
                            "fighter", // Fighting games
                            "ninja", // Ninja games
                            "dragon", // Dragon games
                            "thunder", // Thunder games
                            "star", // Star games
                            "space", // Space games
                            "battle", // Battle games
                            "war", // War games
                            "pc_", // Nintendo PlayChoice games
                            "nes_", // NES games
                            "snes_", // SNES games
                            "md_", // Mega Drive games
                            "tg_", // TurboGrafx games
                            "ngm", // Neo Geo games
                            "mslug", // Metal Slug
                            "ddragon", // Double Dragon
                            "contra", // Contra
                            "gradius", // Gradius
                            "rtype", // R-Type
                        ];

                        let mut is_legitimate_game = false;
                        for pattern in &legitimate_game_patterns {
                            if name_lower.starts_with(pattern) || name_lower.contains(pattern) {
                                is_legitimate_game = true;
                                break;
                            }
                        }

                        // Also check if it's a known game series with year
                        if name_lower.chars().take(4).collect::<String>().parse::<u32>().is_ok() &&
                            name_lower.chars().take(4).collect::<String>().parse::<u32>().unwrap_or(0) >= 1940 &&
                            name_lower.chars().take(4).collect::<String>().parse::<u32>().unwrap_or(0) <= 2100 {
                                is_legitimate_game = true; // Games starting with years
                            }

                            // Filter out IGT (gambling) machines
                            if name_lower.starts_with("igt") && name_lower.len() <= 8 {
                                return false;
                            }

                            if !is_legitimate_game &&
                                name_lower.len() <= 8 &&
                                name_lower.chars().filter(|c| c.is_alphanumeric()).count() == name_lower.len() &&
                                name_lower.chars().filter(|c| c.is_numeric()).count() >= 2 &&
                                !desc_lower.contains("game") &&
                                !desc_lower.contains("arcade") &&
                                !desc_lower.contains("vs.") &&
                                !desc_lower.contains("versus") {
                                    return false;
                                }

                                // Filter out entries with specific numeric patterns (like abc802, noki3210)
                                if name_lower.chars().filter(|c| c.is_numeric()).count() >= 3 &&
                                    (name_lower.contains("abc") || name_lower.contains("noki") ||
                                    name_lower.contains("isbc") || name_lower.contains("pb") ||
                                    name_lower.contains("phc") || name_lower.contains("rc") ||
                                    name_lower.contains("rm") || name_lower.contains("gl") ||
                                    name_lower.contains("ip") || name_lower.contains("nws")) {
                                        return false;
                                    }

                                    // Filter out entries that look like model numbers
                                    if name_lower.chars().filter(|c| c.is_numeric()).count() >= 3 &&
                                        name_lower.contains('-') &&
                                        !desc_lower.contains("game") &&
                                        !desc_lower.contains("arcade") {
                                            return false;
                                        }

                                        // Filter out entries with no controls (usually systems)
                                        if metadata.controls.is_empty() &&
                                            !desc_lower.contains("mahjong") && // Some mahjong games have no controls listed
                                            !desc_lower.contains("quiz") {      // Some quiz games too
                                                return false;
                                            }
                    }
                }

                // Text search - check display name, filename, and description
                let search_text = self.filter_settings.search_text.to_lowercase();
                let text_match = search_text.is_empty() ||
                display.to_lowercase().contains(&search_text) ||
                filename.to_lowercase().contains(&search_text) ||
                self.game_metadata.get(filename)
                .map(|m| m.description.to_lowercase().contains(&search_text))
                .unwrap_or(false);

                if !text_match {
                    return false;
                }

                // Get metadata for additional filtering
                if let Some(metadata) = self.game_metadata.get(filename) {
                    // Year filter
                    if !self.filter_settings.year_from.is_empty() || !self.filter_settings.year_to.is_empty() {
                        if let Ok(year) = metadata.year.parse::<u32>() {
                            if !self.filter_settings.year_from.is_empty() {
                                if let Ok(from_year) = self.filter_settings.year_from.parse::<u32>() {
                                    if year < from_year {
                                        return false;
                                    }
                                }
                            }
                            if !self.filter_settings.year_to.is_empty() {
                                if let Ok(to_year) = self.filter_settings.year_to.parse::<u32>() {
                                    if year > to_year {
                                        return false;
                                    }
                                }
                            }
                        } else if !metadata.year.is_empty() {
                            // If year can't be parsed but filter is set, exclude
                            return false;
                        }
                    }

                    // Manufacturer filter
                    if !self.filter_settings.manufacturer.is_empty() {
                        if metadata.manufacturer != self.filter_settings.manufacturer {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect();

            let row_height = 22.0;
            egui::ScrollArea::vertical().show_rows(
                ui,
                row_height,
                filtered_roms.len(),
                                                   |ui, row_range| {
                                                       for row in row_range {
                                                           let (_, filename) = &filtered_roms[row];
                                                           let is_selected = self.selected_rom.as_deref() == Some(filename);

                                                           let clean_title = self
                                                           .game_metadata
                                                           .get(filename)
                                                           .map(|meta| meta.description.as_str())
                                                           .unwrap_or(filename);
                                                           let year = self
                                                           .game_metadata
                                                           .get(filename)
                                                           .map(|m| m.year.as_str())
                                                           .unwrap_or("");
                                                           let manuf = self
                                                           .game_metadata
                                                           .get(filename)
                                                           .map(|m| m.manufacturer.as_str())
                                                           .unwrap_or("");

                                                           ui.horizontal(|ui| {
                                                               let response = ui.add_sized([400.0, row_height], egui::SelectableLabel::new(is_selected, clean_title));
                                                               if response.clicked() {
                                                                   self.selected_rom = Some(filename.clone());
                                                                   self.art_texture = None;
                                                                   self.screenshot = None;
                                                                   self.texture_handle = None;
                                                               }
                                                               if response.double_clicked() {
                                                                   launch_rom(filename, &self.rom_dirs, &self.extra_rom_dirs, &self.mame_executable);
                                                               }
                                                               ui.add_sized([100.0, row_height], egui::Label::new(filename));
                                                               ui.add_sized([60.0, row_height], egui::Label::new(year));
                                                               ui.add_sized([200.0, row_height], egui::Label::new(manuf));
                                                           });
                                                           ui.separator();
                                                       }
                                                   }
            );

            ui.separator();

            if let Some(rx) = &self.roms_tx {
                if let Ok(loaded_roms) = rx.try_recv() {
                    self.roms = loaded_roms;
                    self.roms_loading = false;
                    self.roms_tx = None;
                }
            }

            // Launch button in main panel
            ui.horizontal(|ui| {
                if ui.button("Launch Selected ROM").clicked() {
                    if let Some(rom_name) = &self.selected_rom {
                        launch_rom(rom_name, &self.rom_dirs, &self.extra_rom_dirs, &self.mame_executable);
                    }
                }

                if let Some(selected) = &self.selected_rom {
                    ui.label(format!("Selected: {}", selected));
                }
            });
        });

        // ---- Bottom Status Bar Panel ----
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Filter status
                let total_roms = self.roms.len();

                // Calculate filtered count using the same logic as the main filter
                let filtered_roms_count = self.roms.iter()
                .filter(|(_, filename)| {
                    // Apply the same filtering logic as in the main list
                    if let Some(metadata) = self.game_metadata.get(filename) {
                        let desc_lower = metadata.description.to_lowercase();
                        let name_lower = filename.to_lowercase();

                        // Content filtering
                        if self.filter_settings.hide_mahjong {
                            if desc_lower.contains("mahjong") || desc_lower.contains("mah-jong") ||
                                desc_lower.contains("mahjongg") || name_lower.contains("mahjong") ||
                                name_lower.starts_with("mj") || name_lower.starts_with("maj") {
                                    return false;
                                }
                        }

                        if self.filter_settings.hide_adult {
                            let adult_keywords = ["nude", "naked", "strip", "adult", "sexy", "erotic",
                            "porn", "xxx", "sex", "topless", "bottomless", "peep"];
                            for keyword in &adult_keywords {
                                if desc_lower.contains(keyword) || name_lower.contains(keyword) {
                                    return false;
                                }
                            }
                        }

                        if self.filter_settings.hide_casino {
                            let casino_keywords = ["poker", "blackjack", "slot", "casino", "gambling"];
                            for keyword in &casino_keywords {
                                if desc_lower.contains(keyword) || name_lower.contains(keyword) {
                                    return false;
                                }
                            }
                        }

                        // Non-game filtering
                        if self.filter_settings.hide_non_games {
                            if metadata.is_device || metadata.is_bios || metadata.is_mechanical || metadata.runnable {
                                return false;
                            }
                        }
                    }

                    // Text search
                    let search_text = self.filter_settings.search_text.to_lowercase();
                    if !search_text.is_empty() {
                        let matches = filename.to_lowercase().contains(&search_text) ||
                        self.game_metadata.get(filename)
                        .map(|m| m.description.to_lowercase().contains(&search_text))
                        .unwrap_or(false);
                        if !matches {
                            return false;
                        }
                    }

                    true
                })
                .count();

                let status_text = if self.filter_settings.search_text.is_empty() &&
                self.filter_settings.year_from.is_empty() &&
                self.filter_settings.year_to.is_empty() &&
                self.filter_settings.manufacturer.is_empty() {
                    format!("Showing {} games in collection", total_roms)
                } else {
                    format!("Filtered: {} games", filtered_roms_count)
                };
                ui.label(status_text);

                ui.separator();

                // MAME statistics
                if self.total_games_count > 0 {
                    ui.label(format!("{} | Total supported: {} games ({} working, {}% playable)",
                                     self.mame_version,
                                     self.total_games_count,
                                     self.working_games_count,
                                     (self.working_games_count as f32 / self.total_games_count as f32 * 100.0) as u32));
                } else if !self.mame_version.is_empty() {
                    ui.label(&self.mame_version);
                } else {
                    ui.label("No MAME executable configured");
                }

                // Add space to right-align the next items
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.roms_loading {
                        ui.spinner();
                        ui.label("Loading ROMs...");
                    }
                });
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    // Load icon for the application
    let icon_data = if let Ok(image) = image::open("assets/mame-frontend-icon.png") {
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
        "MAME Launcher",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp::default()))
        }),
    )
}
