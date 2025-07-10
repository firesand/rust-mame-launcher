use std::collections::HashMap;
use crate::models::{GameMetadata, GameStats, SortColumn, SortDirection, RomStatus};

pub type SortableRom = (String, String, bool, bool); // (display_name, rom_name, is_clone, has_clones)

pub fn sort_rom_list(
    roms: &mut Vec<SortableRom>,
    metadata: &HashMap<String, GameMetadata>,
    game_stats: &HashMap<String, GameStats>,
    sort_column: SortColumn,
    sort_direction: SortDirection,
) {
    roms.sort_by(|a, b| {
        let (display_a, rom_a, _, _) = a;
        let (display_b, rom_b, _, _) = b;

        let meta_a = metadata.get(rom_a);
        let meta_b = metadata.get(rom_b);
        let stats_a = game_stats.get(rom_a);
        let stats_b = game_stats.get(rom_b);

        let ordering = match sort_column {
            SortColumn::Title => display_a.to_lowercase().cmp(&display_b.to_lowercase()),
            SortColumn::RomName => rom_a.to_lowercase().cmp(&rom_b.to_lowercase()),
            SortColumn::Year => {
                let year_a = meta_a.map(|m| m.year.as_str()).unwrap_or("");
                let year_b = meta_b.map(|m| m.year.as_str()).unwrap_or("");
                year_a.cmp(year_b)
            }
            SortColumn::Manufacturer => {
                let manuf_a = meta_a.map(|m| m.manufacturer.as_str()).unwrap_or("");
                let manuf_b = meta_b.map(|m| m.manufacturer.as_str()).unwrap_or("");
                manuf_a.to_lowercase().cmp(&manuf_b.to_lowercase())
            }
            SortColumn::Status => {
                let status_a = meta_a.map(|m| m.get_status()).unwrap_or(RomStatus::NotWorking);
                let status_b = meta_b.map(|m| m.get_status()).unwrap_or(RomStatus::NotWorking);
                (status_a as u8).cmp(&(status_b as u8))
            }
            SortColumn::PlayCount => {
                let count_a = stats_a.map(|s| s.play_count).unwrap_or(0);
                let count_b = stats_b.map(|s| s.play_count).unwrap_or(0);
                count_a.cmp(&count_b)
            }
            SortColumn::LastPlayed => {
                let last_a = stats_a.and_then(|s| s.last_played.as_ref());
                let last_b = stats_b.and_then(|s| s.last_played.as_ref());
                last_a.cmp(&last_b)
            }
        };

        match sort_direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    });
}
