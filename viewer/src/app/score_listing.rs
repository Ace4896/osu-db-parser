use std::collections::HashMap;

use osu_db_parser::prelude::*;

/// A view for displaying score listing details.
#[derive(Default)]
pub struct ScoreListingView {
    score_listing: Option<ScoreListing>,
    // displayed_scores: HashMap<usize, ScoreDetailsWindow>,
}

impl ScoreListingView {
    /// Loads a new score listing.
    pub fn load_score_listing(&mut self, score_listing: ScoreListing) {
        self.score_listing = Some(score_listing);
        self.clear_displayed_scores();
    }

    /// Clears the list of displayed scores.
    pub fn clear_displayed_scores(&mut self) {
        // self.displayed_scores.clear();
    }

    pub fn view(
        &mut self,
        ctx: &egui::Context,
        beatmap_listing: &BeatmapListing,
        md5_mapping: &HashMap<String, usize>,
    ) {
        // TODO: Windows for displayed score details
        // TODO: Nice way to display the "leaderboard"? Maybe show this in the right side panel

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Score Listing");

            if let Some(score_listing) = &self.score_listing {
                ui.horizontal(|ui| {
                    ui.label("Version");
                    ui.label(score_listing.version.to_string());
                });

                // TODO: Beatmaps using MD5 mapping
            } else {
                ui.label("No score listing loaded...");
            }
        });
    }
}
