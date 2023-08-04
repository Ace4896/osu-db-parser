use std::collections::HashMap;

use egui::Id;
use osu_db_parser::prelude::*;

use super::{beatmap_details::BeatmapDetailsWindow, optional_string};

/// A view for displaying collection listing details.
#[derive(Default)]
pub struct CollectionListingView {
    collection_listing: Option<CollectionListing>,
    displayed_beatmaps: HashMap<usize, BeatmapDetailsWindow>,
}

impl CollectionListingView {
    /// Loads a collection listing into this view.
    pub fn load_collection_listing(&mut self, collection_listing: CollectionListing) {
        self.collection_listing = Some(collection_listing);
        self.clear_displayed_beatmaps();
    }

    /// Clears the list of displayed beatmaps. Needs to be called when the beatmap listing changes.
    pub fn clear_displayed_beatmaps(&mut self) {
        self.displayed_beatmaps.clear();
    }

    /// Renders the collection listing view using the specified beatmap listing details.
    pub fn view(
        &mut self,
        ctx: &egui::Context,
        beatmap_listing: &BeatmapListing,
        md5_mapping: &HashMap<String, usize>,
    ) {
        let base_window_id = Id::new("c_beatmap_details");
        self.displayed_beatmaps.retain(|_, window| window.visible);

        for (i, beatmap_details_window) in self.displayed_beatmaps.iter_mut() {
            if let Some(beatmap) = beatmap_listing.beatmaps.get(*i) {
                beatmap_details_window.view(ctx, base_window_id.with(*i), beatmap);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Collection Listing");

            if let Some(collection_listing) = &self.collection_listing {
                // Version Details
                ui.horizontal(|ui| {
                    ui.label("Version");
                    ui.label(collection_listing.version.to_string());
                });

                // Beatmaps in Collections
                for (collection_index, collection) in
                    collection_listing.collections.iter().enumerate()
                {
                    let collection_id = Id::new(collection_index);

                    egui::CollapsingHeader::new(optional_string(&collection.name))
                        .id_source(collection_id)
                        .show(ui, |ui| {
                            for md5 in collection
                                .beatmap_md5s
                                .iter()
                                .filter_map(|md5| md5.as_ref())
                            {
                                if let Some((beatmap_index, beatmap)) = md5_mapping
                                    .get(md5)
                                    .map(|i| beatmap_listing.beatmaps.get(*i).map(|b| (i, b)))
                                    .flatten()
                                {
                                    let name = format!(
                                        "{} - {} [{}]",
                                        &beatmap.artist_name.clone().unwrap_or_default(),
                                        &beatmap.song_title.clone().unwrap_or_default(),
                                        &beatmap.difficulty.clone().unwrap_or_default()
                                    );

                                    if ui.button(&name).clicked() {
                                        self.displayed_beatmaps.insert(
                                            *beatmap_index,
                                            BeatmapDetailsWindow {
                                                title: name,
                                                visible: true,
                                            },
                                        );
                                    }
                                } else {
                                    ui.add_enabled(
                                        false,
                                        egui::Button::new(format!("Unknown (MD5: {})", md5)),
                                    );
                                }
                            }
                        });
                }
            } else {
                ui.label("No collection listing loaded...");
            }
        });
    }
}
