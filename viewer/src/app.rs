use std::collections::HashMap;

use egui::Id;
use osu_db_parser::{flagset, prelude::*};

use crate::widgets::file_dialog::FileDialog;

use self::{
    beatmap_listing::BeatmapListingView, collection_listing::CollectionListingView,
    replays::ReplaysView, score_details::ScoreDetailsWindow,
};

mod beatmap_details;
mod beatmap_listing;
mod collection_listing;
mod replays;
mod score_details;

/// Holds the state for the main application.
pub struct MainApp {
    // File Loading
    file_dialog: FileDialog,
    pending_file_operation: Option<FileOperation>,

    // Views
    current_view: ViewType,
    beatmap_listing: BeatmapListingView,
    collection_listing: CollectionListingView,
    replays: ReplaysView,

    // MD5 Lookups
    beatmaps: HashMap<String, BeatmapEntry>,
    scores: HashMap<String, Vec<ScoreReplay>>,
}

/// Represents the different 'tabs' that can be navigated to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewType {
    BeatmapListing,
    CollectionListing,
    Replays,
}

/// Represents a file operation requested by the user.
#[derive(Clone, Copy, Debug)]
enum FileOperation {
    GetBeatmapListing,
    GetCollectionListing,
    GetScoreListing,
    GetReplay,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            file_dialog: FileDialog::default(),
            pending_file_operation: None,

            current_view: ViewType::BeatmapListing,
            beatmap_listing: BeatmapListingView::default(),
            collection_listing: CollectionListingView::default(),
            replays: ReplaysView::default(),

            beatmaps: HashMap::new(),
            scores: HashMap::new(),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_for_files();
        self.menu_bar(ctx, frame);

        // Determine which view to show
        match self.current_view {
            ViewType::BeatmapListing => self.beatmap_listing.view(ctx, &self.scores),
            ViewType::CollectionListing => {
                self.collection_listing
                    .view(ctx, &self.beatmaps, &self.scores)
            }
            ViewType::Replays => self.replays.view(ctx),
        }
    }
}

impl MainApp {
    /// Checks if we are waiting for a file and attempts to parse it if it has been loaded.
    fn check_for_files(&mut self) {
        if let Some(file_operation) = self.pending_file_operation {
            if let Some(data) = self.file_dialog.get() {
                match file_operation {
                    FileOperation::GetBeatmapListing => match BeatmapListing::from_bytes(&data) {
                        Ok(beatmap_listing) => {
                            // Setup the MD5 mapping for the loaded beatmaps
                            self.beatmaps = beatmap_listing
                                .beatmaps
                                .iter()
                                .filter_map(|b| b.md5.as_ref().map(|md5| (md5.clone(), b.clone())))
                                .collect();

                            // Update any window titles for the replays view
                            self.replays.update_replay_titles(&self.beatmaps);

                            // Load the beatmap listing and change views
                            self.beatmap_listing.load_beatmap_listing(beatmap_listing);
                            self.current_view = ViewType::BeatmapListing;
                        }
                        Err(e) => log::warn!("Unable to open beatmap listing: {}", e),
                    },
                    FileOperation::GetCollectionListing => {
                        match CollectionListing::from_bytes(&data) {
                            Ok(collection_listing) => {
                                self.collection_listing
                                    .load_collection_listing(collection_listing);
                                self.current_view = ViewType::CollectionListing;
                            }
                            Err(e) => log::warn!("Unable to open collection listing: {}", e),
                        }
                    }
                    FileOperation::GetScoreListing => match ScoreListing::from_bytes(&data) {
                        Ok(score_listing) => {
                            log::info!(
                                "Successfully loaded scores.db (version: {})",
                                score_listing.version
                            );

                            // Setup the MD5 mapping for the loaded scores
                            self.scores = score_listing
                                .beatmap_scores
                                .into_iter()
                                .filter_map(|s| s.md5.map(|md5| (md5, s.scores)))
                                .collect();

                            // Order each beatmap's scores by descending score, then ascending date
                            for beatmap_scores in self.scores.values_mut() {
                                beatmap_scores.sort_unstable_by(|a, b| {
                                    b.score
                                        .cmp(&a.score)
                                        .then_with(|| a.timestamp.cmp(&b.timestamp))
                                });
                            }
                        }
                        Err(e) => log::warn!("Unable to open score listing: {}", e),
                    },
                    FileOperation::GetReplay => match ScoreReplay::from_bytes(&data) {
                        Ok(replay) => {
                            log::info!(
                                "Successfully loaded .osr replay (version: {})",
                                replay.version
                            );

                            self.replays.load_replay(replay, &self.beatmaps);
                            self.current_view = ViewType::Replays;
                        }
                        Err(e) => log::warn!("Unable to open replay file: {}", e),
                    },
                }

                self.pending_file_operation = None;
            }
        }
    }

    /// Renders the top panel showing the menu bar.
    fn menu_bar(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    use FileOperation::*;

                    if ui.button("Open osu.db...").clicked() {
                        self.pending_file_operation = Some(GetBeatmapListing);
                        self.file_dialog.open();
                        ui.close_menu();
                    }

                    if ui.button("Open collection.db...").clicked() {
                        self.pending_file_operation = Some(GetCollectionListing);
                        self.file_dialog.open();
                        ui.close_menu();
                    }

                    if ui.button("Open scores.db...").clicked() {
                        self.pending_file_operation = Some(GetScoreListing);
                        self.file_dialog.open();
                        ui.close_menu();
                    }

                    if ui.button("Open .osr replay...").clicked() {
                        self.pending_file_operation = Some(GetReplay);
                        self.file_dialog.open();
                        ui.close_menu();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();

                        if ui.button("Close").clicked() {
                            _frame.close();
                        }
                    }
                });

                ui.separator();

                ui.selectable_value(
                    &mut self.current_view,
                    ViewType::BeatmapListing,
                    "Beatmap Listing",
                );

                ui.selectable_value(
                    &mut self.current_view,
                    ViewType::CollectionListing,
                    "Collection Listing",
                );

                ui.selectable_value(&mut self.current_view, ViewType::Replays, "Replays");
            });
        });
    }
}

/// Opens a beatmap link in the browser.
fn open_beatmap_in_browser(beatmap: &BeatmapEntry) {
    // Fields to populate are:
    // - Beatmapset ID
    // - Gameplay Mode - #osu, #taiko, #fruits, #mania
    // - Difficulty ID
    let url = format!(
        "https://osu.ppy.sh/beatmapsets/{}{}/{}",
        beatmap.beatmap_id,
        match beatmap.gameplay_mode {
            GameplayMode::Standard => "#osu",
            GameplayMode::Taiko => "#taiko",
            GameplayMode::Catch => "#fruits",
            GameplayMode::Mania => "#mania",
        },
        beatmap.difficulty_id
    );

    if let Err(e) = webbrowser::open(&url) {
        log::error!("Unable to open beatmap link '{}': {}", &url, e);
    }
}

/// Opens a score details link in the browser.
fn open_score_in_browser(score: &ScoreReplay) {
    // Fields to populate are:
    // - Gameplay Mode - osu, taiko, fruits, mania
    // - Online Score ID
    let url = format!("https://osu.ppy.sh/scores/{}/{}",
        match score.gameplay_mode {
            GameplayMode::Standard => "osu",
            GameplayMode::Taiko => "taiko",
            GameplayMode::Catch => "fruits",
            GameplayMode::Mania => "mania",
        },
        score.online_score_id
    );

    if let Err(e) = webbrowser::open(&url) {
        log::error!("Unable to open score link '{}': {}", &url, e);
    }
}

/// Renders a flagset as a more readable string.
fn flagset_string<F: flagset::Flags>(flags: flagset::FlagSet<F>) -> String {
    flags
        .into_iter()
        .map(|f| format!("{:?}", f))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Renders a mods flagset as a more readable string.
fn mods_string(mods: flagset::FlagSet<Mods>) -> String {
    if mods.is_empty() {
        "NoMod".to_string()
    } else {
        flagset_string(mods)
    }
}

/// Renders an unsigned u32 value that acts as -1 when it is the maximum value.
fn maybe_signed_u32(val: u32) -> egui::WidgetText {
    if val == 0xFFFFFFFF {
        "-1".into()
    } else {
        val.to_string().into()
    }
}

/// Renders an unsigned u64 value that acts as -1 when it is the maximum value.
fn maybe_signed_u64(val: u64) -> egui::WidgetText {
    if val == u64::MAX {
        "-1".into()
    } else {
        val.to_string().into()
    }
}

/// Renders an optional string.
fn optional_string<T: std::fmt::Display>(value: &Option<T>) -> egui::WidgetText {
    if let Some(v) = value {
        v.to_string().into()
    } else {
        egui::RichText::new("N/A").italics().into()
    }
}

/// Renders a leaderboard of scores for a particular beatmap.
/// Assumes that the score values are sorted in descending order.
fn leaderboard(
    ui: &mut egui::Ui,
    scores: &Vec<ScoreReplay>,
    score_windows: &mut HashMap<String, ScoreDetailsWindow>,
) {
    let row_height = ui.text_style_height(&egui::TextStyle::Body);

    egui::ScrollArea::both()
        .auto_shrink([false, true])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show_rows(ui, row_height, scores.len(), |ui, row_range| {
            for i in row_range {
                // Replays should have an MD5 hash
                let details = &scores[i];
                if let Some(replay_md5) = &details.replay_md5 {
                    // TODO: Mod combination
                    let label = format!(
                        "({}) {}: {} - {} - {} {:.02}%",
                        details.gameplay_mode,
                        i + 1,
                        details.grade(),
                        details.player_name.clone().unwrap_or_default(),
                        details.score,
                        details.accuracy()
                    );

                    if ui.selectable_label(false, &label).clicked() {
                        score_windows.insert(
                            replay_md5.to_string(),
                            ScoreDetailsWindow {
                                id: Id::new("score_details").with(i),
                                title: label,
                                visible: true,
                                data: details.clone(),
                            },
                        );
                    };
                }
            }
        });
}
