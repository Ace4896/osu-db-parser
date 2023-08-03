use egui::Id;
use osu_db_parser::prelude::*;

use crate::widgets::file_dialog::FileDialog;

/// Holds the state for the main application.
pub struct MainApp {
    next_id: u32,
    file_dialog: FileDialog,
    pending_file_operation: Option<FileOperation>,

    open_beatmap_listings: Vec<OpenBeatmapListing>,
}

/// Represents a file operation requested by the user.
#[derive(Clone, Copy, Debug)]
pub enum FileOperation {
    GetBeatmapListing,
    GetCollectionListing { id: Id },
    GetScoreListing { id: Id },
    GetReplay { id: Id },
}

/// Holds the state for any open beatmap listings.
pub struct OpenBeatmapListing {
    id: Id,
    title: String,
    visible: bool,

    beatmap_listing: BeatmapListing,
    collection_listings: Vec<CollectionListing>,
    score_listings: Vec<ScoreListing>,
    replays: Vec<ScoreReplay>,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            next_id: 0,
            file_dialog: FileDialog::default(),
            pending_file_operation: None,

            open_beatmap_listings: Vec::new(),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_for_files();
        self.side_panel(ctx);

        self.beatmap_listing_windows(ctx);
    }
}

impl MainApp {
    /// Sets up a new beatmap listing that was just opened.
    fn setup_beatmap_listing(&mut self, beatmap_listing: BeatmapListing) -> OpenBeatmapListing {
        let id = self.next_id;
        let title = format!("Beatmap Listing #{}", id);

        self.next_id += 1;

        OpenBeatmapListing {
            id: Id::new(id),
            title,
            visible: true,
            beatmap_listing,
            collection_listings: Vec::new(),
            score_listings: Vec::new(),
            replays: Vec::new(),
        }
    }

    /// Finds the opened database with the specified ID.
    fn find_opened_database(&mut self, id: Id) -> Option<&mut OpenBeatmapListing> {
        self.open_beatmap_listings.iter_mut().find(|db| db.id == id)
    }

    /// Checks if we are waiting for a file and attempts to parse it if it has been loaded.
    fn check_for_files(&mut self) {
        if let Some(file_operation) = self.pending_file_operation {
            if let Some(data) = self.file_dialog.get() {
                match file_operation {
                    FileOperation::GetBeatmapListing => match BeatmapListing::from_bytes(&data) {
                        Ok(beatmap_listing) => {
                            let opened_beatmap_listing =
                                self.setup_beatmap_listing(beatmap_listing);
                            self.open_beatmap_listings.push(opened_beatmap_listing);
                        }
                        Err(e) => log::warn!("Unable to open beatmap listing: {}", e),
                    },
                    FileOperation::GetCollectionListing {
                        id: open_database_id,
                    } => {
                        if let Some(opened_database) = self.find_opened_database(open_database_id) {
                            match CollectionListing::from_bytes(&data) {
                                Ok(collection_listing) => {
                                    opened_database.collection_listings.push(collection_listing)
                                }
                                Err(e) => log::warn!("Unable to open collection listing: {}", e),
                            }
                        }
                    }
                    FileOperation::GetScoreListing {
                        id: open_database_id,
                    } => {
                        if let Some(opened_database) = self.find_opened_database(open_database_id) {
                            match ScoreListing::from_bytes(&data) {
                                Ok(score_listing) => {
                                    opened_database.score_listings.push(score_listing)
                                }
                                Err(e) => log::warn!("Unable to open score listing: {}", e),
                            }
                        }
                    }
                    FileOperation::GetReplay {
                        id: open_database_id,
                    } => {
                        if let Some(opened_database) = self.find_opened_database(open_database_id) {
                            match ScoreReplay::from_bytes(&data) {
                                Ok(replay) => opened_database.replays.push(replay),
                                Err(e) => log::warn!("Unable to open replay file: {}", e),
                            }
                        }
                    }
                }

                self.pending_file_operation = None;
            }
        }
    }

    /// Renders the side bar which displays any opened beatmap listings.
    fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading("Open Databases"));

            if ui.button("Open osu.db...").clicked() {
                if self.pending_file_operation.is_none() {
                    self.pending_file_operation = Some(FileOperation::GetBeatmapListing);
                    self.file_dialog.open();
                }
            }

            for open_beatmap_listing in self.open_beatmap_listings.iter_mut() {
                ui.collapsing(&open_beatmap_listing.title, |ui| {
                    // Open the beatmap listing window (or focus it if it's already open)
                    if ui.button("View Details").clicked() {
                        if open_beatmap_listing.visible {
                            ui.memory_mut(|memory| memory.request_focus(open_beatmap_listing.id));
                        } else {
                            open_beatmap_listing.visible = true;
                        }
                    }

                    // Headers for collections, scores and replays
                    ui.collapsing("Collection Listings", |ui| {
                        if ui.button("Open collection.db...").clicked() {
                            if self.pending_file_operation.is_none() {
                                self.pending_file_operation =
                                    Some(FileOperation::GetCollectionListing {
                                        id: open_beatmap_listing.id,
                                    });

                                self.file_dialog.open();
                            }
                        }

                        for (i, _) in open_beatmap_listing
                            .collection_listings
                            .iter_mut()
                            .enumerate()
                        {
                            let _ = ui.button(&format!("Collection Listing #{}", i));
                        }
                    });

                    ui.collapsing("Score Listings", |ui| {
                        if ui.button("Open scores.db...").clicked() {
                            if self.pending_file_operation.is_none() {
                                self.pending_file_operation =
                                    Some(FileOperation::GetScoreListing {
                                        id: open_beatmap_listing.id,
                                    });

                                self.file_dialog.open();
                            }
                        }

                        for (i, _) in open_beatmap_listing.score_listings.iter_mut().enumerate() {
                            let _ = ui.button(&format!("Score Listing #{}", i));
                        }
                    });

                    ui.collapsing("Replays", |ui| {
                        if ui.button("Open .osr replay...").clicked() {
                            if self.pending_file_operation.is_none() {
                                self.pending_file_operation = Some(FileOperation::GetReplay {
                                    id: open_beatmap_listing.id,
                                });

                                self.file_dialog.open();
                            }
                        }

                        for (i, _) in open_beatmap_listing.replays.iter_mut().enumerate() {
                            let _ = ui.button(&format!("Replay #{}", i));
                        }
                    });
                });
            }
        });
    }

    /// Renders the beatmap listing windows.
    fn beatmap_listing_windows(&mut self, ctx: &egui::Context) {
        for open_beatmap_listing in self.open_beatmap_listings.iter_mut().filter(|b| b.visible) {
            egui::Window::new(&open_beatmap_listing.title)
                .id(open_beatmap_listing.id)
                .open(&mut open_beatmap_listing.visible)
                .show(ctx, |ui| {});
        }
    }
}
