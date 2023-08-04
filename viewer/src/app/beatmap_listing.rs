use std::collections::HashMap;

use egui::Id;
use osu_db_parser::prelude::*;

use super::beatmap_details::BeatmapDetailsWindow;

/// A view for displaying beatmap listing details.
#[derive(Default)]
pub struct BeatmapListingView {
    pub beatmap_listing: Option<BeatmapListing>,
    pub md5_mapping: HashMap<String, usize>,

    displayed_beatmaps: HashMap<usize, BeatmapDetailsWindow>,
}

impl BeatmapListingView {
    /// Loads a beatmap listing into this view.
    pub fn load_beatmap_listing(&mut self, beatmap_listing: BeatmapListing) {
        // Setup MD5 mapping
        self.md5_mapping = beatmap_listing
            .beatmaps
            .iter()
            .enumerate()
            .filter_map(|(i, beatmap)| beatmap.md5.as_ref().map(|md5| (md5.to_string(), i)))
            .collect();

        // Load beatmap listing and clear any previously displayed beatmaps (as they are now invalid)
        self.beatmap_listing = Some(beatmap_listing);
        self.displayed_beatmaps.clear();
    }

    /// Renders the beatmap listing view.
    pub fn view(&mut self, ctx: &egui::Context) {
        let base_window_id = Id::new("b_beatmap_details");
        self.displayed_beatmaps.retain(|_, window| window.visible);

        for (i, beatmap_details_window) in self.displayed_beatmaps.iter_mut() {
            if let Some(beatmap) = self
                .beatmap_listing
                .as_ref()
                .and_then(|listing| listing.beatmaps.get(*i))
            {
                beatmap_details_window.view(ctx, base_window_id.with(*i), beatmap);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Beatmap Listing");

            if let Some(beatmap_listing) = &self.beatmap_listing {
                // Base Details
                egui::Grid::new("base_details").show(ui, |ui| {
                    ui.label("Version");
                    ui.label(beatmap_listing.version.to_string());
                    ui.end_row();

                    ui.label("Folder Count");
                    ui.label(beatmap_listing.folder_count.to_string());
                    ui.end_row();

                    ui.label("Account Unlocked?");
                    ui.label(beatmap_listing.account_unlocked.to_string());
                    ui.end_row();

                    ui.label("Account Unlock Date");
                    ui.label(beatmap_listing.account_unlock_date.to_string());
                    ui.end_row();

                    ui.label("Player Name");
                    ui.label(beatmap_listing.player_name.clone().unwrap_or_default());
                    ui.end_row();

                    // TODO: Formatting for flag sets like this
                    ui.label("User Permissions");
                    ui.label(format!("{:?}", beatmap_listing.user_permissions));
                    ui.end_row();
                });

                // Beatmaps
                ui.collapsing("Beatmaps", |ui| {
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);

                    egui::ScrollArea::both()
                        .auto_shrink([false, true])
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show_rows(
                            ui,
                            row_height,
                            beatmap_listing.beatmaps.len(),
                            |ui, row_range| {
                                for i in row_range {
                                    let beatmap = &beatmap_listing.beatmaps[i];
                                    let header = format!(
                                        "{} - {} [{}]",
                                        &beatmap.artist_name.clone().unwrap_or_default(),
                                        &beatmap.song_title.clone().unwrap_or_default(),
                                        &beatmap.difficulty.clone().unwrap_or_default()
                                    );

                                    if ui.small_button(&header).clicked() {
                                        self.displayed_beatmaps.insert(
                                            i,
                                            BeatmapDetailsWindow {
                                                title: header,
                                                visible: true,
                                            },
                                        );
                                    }
                                }
                            },
                        );
                });
            } else {
                ui.label("No beatmap listing loaded...");
            }
        });
    }
}
