use std::collections::HashMap;

use egui::Id;
use osu_db_parser::prelude::*;

use super::{
    beatmap_details::BeatmapDetailsWindow, open_beatmap_in_browser,
    score_details::ScoreDetailsWindow,
};

/// A view for displaying collection listing details.
#[derive(Default)]
pub struct CollectionListingView {
    data: Option<CollectionListing>,
    selected_collection: Option<usize>,
    selected_beatmap_md5: Option<String>,

    beatmap_windows: HashMap<String, BeatmapDetailsWindow>,
    score_windows: HashMap<String, ScoreDetailsWindow>,
}

impl CollectionListingView {
    /// Loads a collection listing into this view.
    pub fn load_collection_listing(&mut self, collection_listing: CollectionListing) {
        self.data = Some(collection_listing);
        self.selected_collection = None;
        self.selected_beatmap_md5 = None;
    }

    /// Renders the collection listing view using the specified beatmap listing details.
    pub fn view(
        &mut self,
        ctx: &egui::Context,
        beatmaps: &HashMap<String, BeatmapEntry>,
        scores: &HashMap<String, Vec<ScoreReplay>>,
    ) {
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

        // Render the central panel showing collections + beatmaps
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Collection Listing");

            if let Some(collection_listing) = &self.data {
                // Version Details
                ui.horizontal(|ui| {
                    ui.label("Version");
                    ui.label(collection_listing.version.to_string());
                });

                // Available Collections
                egui::ComboBox::from_id_source("available_collections")
                    .width(ui.available_width())
                    .selected_text(
                        self.selected_collection
                            .and_then(|i| collection_listing.collections.get(i))
                            .and_then(|collection| collection.name.as_ref().map(|n| n.as_str()))
                            .unwrap_or_else(|| "Select collection..."),
                    )
                    .show_ui(ui, |ui| {
                        for (i, collection) in collection_listing.collections.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.selected_collection,
                                Some(i),
                                collection.name.clone().unwrap_or_default(),
                            );
                        }
                    });

                // Beatmaps in Current Collection
                if let Some(collection) = self
                    .selected_collection
                    .and_then(|i| collection_listing.collections.get(i))
                {
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);

                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show_rows(
                            ui,
                            row_height,
                            collection.beatmap_md5s.len(),
                            |ui, row_range| {
                                // Beatmaps references without an MD5 are invalid - most likely a corrupt DB
                                for i in row_range {
                                    if let Some(md5) = collection.beatmap_md5s[i]
                                        .as_ref()
                                        .filter(|md5| !md5.is_empty())
                                    {
                                        if let Some(beatmap) = beatmaps.get(md5) {
                                            let name = format!(
                                                "{} - {} [{}]",
                                                &beatmap.artist_name.clone().unwrap_or_default(),
                                                &beatmap.song_title.clone().unwrap_or_default(),
                                                &beatmap.difficulty.clone().unwrap_or_default()
                                            );

                                            ui.selectable_value(
                                                &mut self.selected_beatmap_md5,
                                                Some(md5.clone()),
                                                &name,
                                            )
                                            .context_menu(|ui| {
                                                if ui.button("Details").clicked() {
                                                    self.beatmap_windows.insert(
                                                        md5.clone(),
                                                        BeatmapDetailsWindow {
                                                            id: Id::new("c_beatmap_details")
                                                                .with(i),
                                                            title: name,
                                                            visible: true,
                                                            data: beatmap.clone(),
                                                        },
                                                    );

                                                    ui.close_menu();
                                                }

                                                if ui.button("View Beatmap Online").clicked() {
                                                    open_beatmap_in_browser(&beatmap);
                                                    ui.close_menu();
                                                }
                                            });
                                        } else {
                                            ui.add_enabled(
                                                false,
                                                egui::SelectableLabel::new(
                                                    false,
                                                    format!("Unknown (MD5: {})", md5),
                                                ),
                                            );
                                        }
                                    }
                                }
                            },
                        );
                }
            } else {
                ui.label("No collection listing loaded...");
            }
        });
    }
}
