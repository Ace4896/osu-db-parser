use osu_db_parser::prelude::*;

use crate::widgets::file_dialog::FileDialog;

use self::{
    beatmap_listing::BeatmapListingView, collection_listing::CollectionListingView,
    score_listing::ScoreListingView,
};

mod beatmap_details;
mod beatmap_listing;
mod collection_listing;
mod replay;
mod score_details;
mod score_listing;

// TODO: Simplified layout (again):
//
// - Display separate windows for:
//   - Score (from score listing)
//   - Replays (manually opened)

/// Holds the state for the main application.
pub struct MainApp {
    file_dialog: FileDialog,
    pending_file_operation: Option<FileOperation>,

    current_view: ViewType,
    beatmap_listing_view: BeatmapListingView,
    collection_listing_view: CollectionListingView,
    score_listing_view: ScoreListingView,
    replays: Vec<WindowDetails<ScoreReplay>>,
}

/// Represents generic window details for a value.
pub struct WindowDetails<T> {
    visible: bool,
    title: String,
    data: T,
}

/// Represents the different 'tabs' that can be navigated to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewType {
    BeatmapListing,
    CollectionListing,
    ScoreListing,
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
            beatmap_listing_view: BeatmapListingView::default(),
            collection_listing_view: CollectionListingView::default(),
            score_listing_view: ScoreListingView::default(),
            replays: Vec::new(),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_for_files();
        self.menu_bar(ctx, frame);

        match (
            self.current_view,
            &self.beatmap_listing_view.beatmap_listing,
        ) {
            (ViewType::BeatmapListing, _) => self.beatmap_listing_view.view(ctx),
            (ViewType::CollectionListing, Some(beatmap_listing)) => self
                .collection_listing_view
                .view(ctx, beatmap_listing, &self.beatmap_listing_view.md5_mapping),
            (ViewType::ScoreListing, Some(beatmap_listing)) => self.score_listing_view.view(
                ctx,
                beatmap_listing,
                &self.beatmap_listing_view.md5_mapping,
            ),
            (ViewType::Replays, Some(_)) => todo!(),
            _ => {}
        }
    }
}

impl MainApp {
    /// Checks if we are waiting for a file and attempts to parse it if it has been loaded.
    fn check_for_files(&mut self) {
        if let Some(file_operation) = self.pending_file_operation {
            if let Some(data) = self.file_dialog.get() {
                match file_operation {
                    FileOperation::GetBeatmapListing => {
                        match BeatmapListing::from_bytes(&data) {
                            Ok(beatmap_listing) => {
                                // Update the currently displayed values
                                self.beatmap_listing_view
                                    .load_beatmap_listing(beatmap_listing);

                                // Clear any currently displayed beatmaps
                                self.collection_listing_view.clear_displayed_beatmaps();
                            }
                            Err(e) => log::warn!("Unable to open beatmap listing: {}", e),
                        }
                    }
                    FileOperation::GetCollectionListing => {
                        match CollectionListing::from_bytes(&data) {
                            Ok(collection_listing) => {
                                self.collection_listing_view
                                    .load_collection_listing(collection_listing);
                            }
                            Err(e) => log::warn!("Unable to open collection listing: {}", e),
                        }
                    }
                    FileOperation::GetScoreListing => match ScoreListing::from_bytes(&data) {
                        Ok(score_listing) => {
                            self.score_listing_view.load_score_listing(score_listing);
                        }
                        Err(e) => log::warn!("Unable to open score listing: {}", e),
                    },
                    FileOperation::GetReplay => match ScoreReplay::from_bytes(&data) {
                        Ok(replay) => {
                            let title = format!("Replay #{}", self.replays.len());
                            self.replays.push(WindowDetails {
                                visible: true,
                                title,
                                data: replay,
                            });
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

                    ui.add_enabled_ui(self.beatmap_listing_view.beatmap_listing.is_some(), |ui| {
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
                    });

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

                ui.add_enabled_ui(self.beatmap_listing_view.beatmap_listing.is_some(), |ui| {
                    ui.selectable_value(
                        &mut self.current_view,
                        ViewType::CollectionListing,
                        "Collection Listing",
                    );

                    ui.selectable_value(
                        &mut self.current_view,
                        ViewType::ScoreListing,
                        "Score Listing",
                    );

                    ui.selectable_value(&mut self.current_view, ViewType::Replays, "Replays");
                });
            });
        });
    }
}

/// Renders an unsigned value that acts as -1 when it is the maximum value.
fn maybe_signed(val: u32) -> egui::WidgetText {
    if val == 0xFFFFFFFF {
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
