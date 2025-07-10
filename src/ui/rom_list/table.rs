use eframe::egui;
use std::collections::{HashMap, HashSet};
use crate::app::MyApp;
use crate::rom_utils::apply_rom_filters;
use super::sorting::{sort_rom_list, SortableRom};
use super::table_row;

pub fn show_rom_table(app: &mut MyApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let (filtered_roms, virtual_parents, parent_to_clones) = build_filtered_rom_list(app);

    // Calculate row height based on icon size
    let base_row_height = 24.0;
    let row_height = if app.config.show_rom_icons {
        f32::max(base_row_height, app.config.icon_size as f32 + 4.0)
    } else {
        base_row_height
    };

    // Render the ROM list
    egui::ScrollArea::vertical()
    .auto_shrink([false; 2])
    .show_rows(
        ui,
        row_height,
        filtered_roms.len(),
               |ui, row_range| {
                   preload_icons_for_range(app, &filtered_roms, &virtual_parents, row_range.clone());

                   for row in row_range {
                       if let Some(rom_data) = filtered_roms.get(row) {
                           table_row::render_rom_row(
                               app,
                               ui,
                               ctx,
                               rom_data,
                               &virtual_parents,
                               &parent_to_clones,
                               row,
                               row_height,
                           );
                       }
                   }
               }
    );
}

fn build_filtered_rom_list(app: &mut MyApp) -> (Vec<SortableRom>, HashMap<String, String>, HashMap<String, Vec<(String, String)>>) {
    // Build parent-clone relationships
    let (parent_to_clones, clone_to_parent, all_roms_map) = build_parent_clone_maps(app);

    // Find virtual parents
    let virtual_parents = find_virtual_parents(app, &parent_to_clones, &all_roms_map);

    // Log analysis
    log_parent_clone_analysis(app, &parent_to_clones, &clone_to_parent, &virtual_parents);

    // Build and filter display list
    let mut filtered_roms = build_display_list(
        app,
        &parent_to_clones,
        &clone_to_parent,
        &virtual_parents
    );

    // Apply sorting
    sort_rom_list(
        &mut filtered_roms,
        &app.game_metadata,
        &app.config.game_stats,
        app.config.sort_column,
        app.config.sort_direction
    );

    (filtered_roms, virtual_parents, parent_to_clones)
}

fn build_parent_clone_maps(app: &MyApp) -> (
    HashMap<String, Vec<(String, String)>>,
                                            HashMap<String, String>,
                                            HashMap<String, String>
) {
    let mut parent_to_clones: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut clone_to_parent: HashMap<String, String> = HashMap::new();
    let mut all_roms_map: HashMap<String, String> = HashMap::new();

    for (display, rom_name) in &app.roms {
        all_roms_map.insert(rom_name.clone(), display.clone());

        if let Some(metadata) = app.game_metadata.get(rom_name) {
            if let Some(parent) = &metadata.parent {
                clone_to_parent.insert(rom_name.clone(), parent.clone());
                parent_to_clones.entry(parent.clone())
                .or_insert_with(Vec::new)
                .push((display.clone(), rom_name.clone()));
            }
        }
    }

    (parent_to_clones, clone_to_parent, all_roms_map)
}

fn find_virtual_parents(
    app: &MyApp,
    parent_to_clones: &HashMap<String, Vec<(String, String)>>,
                        all_roms_map: &HashMap<String, String>
) -> HashMap<String, String> {
    let mut virtual_parents = HashMap::new();

    for (parent, clones) in parent_to_clones {
        if !all_roms_map.contains_key(parent) && !clones.is_empty() {
            let parent_title = app.game_metadata.get(parent)
            .map(|m| m.description.clone())
            .unwrap_or_else(|| {
                if let Some((_, clone_name)) = clones.first() {
                    if let Some(clone_meta) = app.game_metadata.get(clone_name) {
                        return clone_meta.description.split('(').next()
                        .unwrap_or(parent).trim().to_string() + " (Parent ROM)";
                    }
                }
                format!("{} (Parent ROM)", parent.to_uppercase())
            });

            virtual_parents.insert(parent.clone(), parent_title.clone());
            println!("Created virtual parent: {} -> {}", parent, parent_title);
        }
    }

    virtual_parents
}

fn build_display_list(
    app: &mut MyApp,
    parent_to_clones: &HashMap<String, Vec<(String, String)>>,
                      clone_to_parent: &HashMap<String, String>,
                      virtual_parents: &HashMap<String, String>
) -> Vec<SortableRom> {
    let mut display_list = Vec::new();
    let mut processed = HashSet::new();

    // Collect all entries
    let mut all_entries: Vec<(String, String, bool)> = Vec::new();

    // Add real ROMs
    for (display, rom_name) in &app.roms {
        let is_clone = clone_to_parent.contains_key(rom_name);
        all_entries.push((display.clone(), rom_name.clone(), is_clone));
    }

    // Add virtual parents
    for (parent_name, display_name) in virtual_parents {
        all_entries.push((display_name.clone(), parent_name.clone(), false));
    }

    // Sort entries
    all_entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    // Process entries
    for (display, rom_name, is_clone) in all_entries {
        if processed.contains(&rom_name) {
            continue;
        }

        // Handle clone visibility
        if is_clone {
            let show_this_clone = if let Some(parent) = clone_to_parent.get(&rom_name) {
                *app.expanded_parents.get(parent).unwrap_or(&false) || app.config.filter_settings.show_clones
            } else {
                app.config.filter_settings.show_clones
            };

            if !show_this_clone {
                continue;
            }
        }

        // Apply filters
        if !is_clone && !virtual_parents.contains_key(&rom_name) {
            if !apply_rom_filters(&app.config.filter_settings, &app.game_metadata, &display, &rom_name, &app.config.favorite_games) {
                continue;
            }
        }

        let has_clones = parent_to_clones.contains_key(&rom_name);
        display_list.push((display.clone(), rom_name.clone(), is_clone, has_clones));
        processed.insert(rom_name.clone());

        // Add expanded clones
        if has_clones && *app.expanded_parents.get(&rom_name).unwrap_or(&false) {
            if let Some(clones) = parent_to_clones.get(&rom_name) {
                for (clone_display, clone_name) in clones {
                    if !processed.contains(clone_name) && apply_rom_filters(&app.config.filter_settings, &app.game_metadata, clone_display, clone_name, &app.config.favorite_games) {
                        display_list.push((clone_display.clone(), clone_name.clone(), true, false));
                        processed.insert(clone_name.clone());
                    }
                }
            }
        }
    }

    display_list
}

fn preload_icons_for_range(
    app: &mut MyApp,
    filtered_roms: &[SortableRom],
    virtual_parents: &HashMap<String, String>,
    row_range: std::ops::Range<usize>
) {
    if !app.config.show_rom_icons {
        return;
    }

    let has_icons_path = app.config.extra_asset_dirs.iter()
    .any(|dir| dir.join("icons").exists());

    if has_icons_path {
        let start_idx = row_range.start.saturating_sub(5);
        let end_idx = (row_range.end + 5).min(filtered_roms.len());

        for idx in start_idx..end_idx {
            if let Some((_, rom_name, _, _)) = filtered_roms.get(idx) {
                if !virtual_parents.contains_key(rom_name) {
                    app.queue_icon_load(rom_name.clone());
                }
            }
        }
    }
}

fn log_parent_clone_analysis(
    app: &MyApp,
    parent_to_clones: &HashMap<String, Vec<(String, String)>>,
                             clone_to_parent: &HashMap<String, String>,
                             virtual_parents: &HashMap<String, String>
) {
    println!("\n=== Parent/Clone Analysis ===");
    println!("Total ROMs in collection: {}", app.roms.len());
    println!("Total unique games with clones: {}", parent_to_clones.len());
    println!("Virtual parents created: {}", virtual_parents.len());

    let mut clone_counts: Vec<(String, usize)> = parent_to_clones.iter()
    .map(|(parent, clones)| (parent.clone(), clones.len()))
    .collect();
    clone_counts.sort_by_key(|(_, count)| *count);
    clone_counts.reverse();

    println!("\nTop 10 parents with most clones:");
    for (parent, count) in clone_counts.iter().take(10) {
        println!("  {} has {} clones", parent, count);
    }

    println!("\nTotal parent games with clones: {}", parent_to_clones.len());
    println!("Total clone games: {}", clone_to_parent.len());
}
