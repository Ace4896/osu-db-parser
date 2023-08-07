use std::collections::HashMap;

use egui::Id;
use osu_db_parser::prelude::*;

use super::{beatmap_details::BeatmapDetailsWindow, score_details::ScoreDetailsWindow};

/// A view for displaying beatmap listing details.
#[derive(Default)]
pub struct BeatmapListingView {
    data: Option<BeatmapListing>,
    selected_beatmap_md5: Option<String>,

    beatmap_windows: HashMap<String, BeatmapDetailsWindow>,
    score_windows: HashMap<String, ScoreDetailsWindow>,
}

impl BeatmapListingView {
    /// Loads a beatmap listing into this view.
    pub fn load_beatmap_listing(&mut self, beatmap_listing: BeatmapListing) {
        self.data = Some(beatmap_listing);
        self.selected_beatmap_md5 = None;
    }

    /// Renders the beatmap listing view.
    pub fn view(&mut self, ctx: &egui::Context, scores: &HashMap<String, Vec<ScoreReplay>>) {
        // Unload any closed windows
        self.beatmap_windows.retain(|_, w| w.visible);
        self.score_windows.retain(|_, w| w.visible);

        // Show the remaining windows
        for beatmap_window in self.beatmap_windows.values_mut() {
            beatmap_window.view(ctx);
        }

        for score_window in self.score_windows.values_mut() {
            score_window.view(ctx);
        }

        // Render the left panel showing scores for the selected beatmap
        egui::SidePanel::left("b_beatmap_scores").show_animated(
            ctx,
            self.selected_beatmap_md5.is_some(),
            |ui| {
                ui.heading("Local Scores");

                if let Some(scores) = &self
                    .selected_beatmap_md5
                    .as_ref()
                    .and_then(|md5| scores.get(md5))
                    .filter(|beatmap_scores| !beatmap_scores.is_empty())
                {
                    super::leaderboard(ui, scores, &mut self.score_windows)
                } else {
                    ui.label("No local scores found");
                }
            },
        );

        // Render the central panel showing listing details + beatmaps
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Beatmap Listing");

            if let Some(beatmap_listing) = &self.data {
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
                                    let md5 = beatmap.md5.clone().unwrap_or_default();

                                    // Beatmaps without an MD5 are invalid - most likely a corrupt DB
                                    if !md5.is_empty() {
                                        let header = format!(
                                            "{} - {} [{}]",
                                            &beatmap.artist_name.clone().unwrap_or_default(),
                                            &beatmap.song_title.clone().unwrap_or_default(),
                                            &beatmap.difficulty.clone().unwrap_or_default()
                                        );

                                        ui.selectable_value(
                                            &mut self.selected_beatmap_md5,
                                            Some(beatmap.md5.clone().unwrap_or_default()),
                                            &header,
                                        )
                                        .context_menu(
                                            |ui| {
                                                if ui.button("Details").clicked() {
                                                    self.beatmap_windows.insert(
                                                        md5,
                                                        BeatmapDetailsWindow {
                                                            id: Id::new("b_beatmap_details")
                                                                .with(i),
                                                            title: header,
                                                            visible: true,
                                                            data: beatmap.clone(),
                                                        },
                                                    );

                                                    ui.close_menu();
                                                }
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
