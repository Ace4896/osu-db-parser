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
        // TODO: Side panel which shows the current beatmap details?
        // My original idea is kinda janky, and doesn't work since virtualization removes it after scrolling

        for open_beatmap_listing in self.open_beatmap_listings.iter_mut().filter(|b| b.visible) {
            egui::Window::new(&open_beatmap_listing.title)
                .id(open_beatmap_listing.id)
                .open(&mut open_beatmap_listing.visible)
                .resizable(true)
                .show(ctx, |ui| {
                    let beatmap_listing = &open_beatmap_listing.beatmap_listing;

                    // Side Panel w/ Beatmap Details


                    // Base Details
                    egui::Grid::new(open_beatmap_listing.id.with("base_details")).show(ui, |ui| {
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

                                        let header_id = open_beatmap_listing.id.with(i);

                                        if ui.small_button(&header).clicked() {
                                            beatmap_details_window(
                                                header_id.with("window"),
                                                &header,
                                                beatmap,
                                                ctx,
                                            )
                                        }
                                    }
                                },
                            );
                    });
                });
        }
    }
}

fn beatmap_details_window(id: Id, window_title: &str, beatmap: &BeatmapEntry, ctx: &egui::Context) {
    egui::Window::new(window_title).id(id).show(ctx, |ui| {
        egui::ScrollArea::both()
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                egui::Grid::new(id.with("grid")).show(ui, |ui| {
                    ui.label("Size");
                    ui.label(optional_string(&beatmap.size));
                    ui.end_row();

                    ui.label("Artist Name");
                    ui.label(optional_string(&beatmap.artist_name));
                    ui.end_row();

                    ui.label("Artist Name (Unicode)");
                    ui.label(optional_string(&beatmap.artist_name_unicode));
                    ui.end_row();

                    ui.label("Song Title");
                    ui.label(optional_string(&beatmap.song_title));
                    ui.end_row();

                    ui.label("Song Title (Unicode)");
                    ui.label(optional_string(&beatmap.song_title_unicode));
                    ui.end_row();

                    ui.label("Creator");
                    ui.label(optional_string(&beatmap.creator_name));
                    ui.end_row();

                    ui.label("Difficulty");
                    ui.label(optional_string(&beatmap.difficulty));
                    ui.end_row();

                    ui.label("Audio Filename");
                    ui.label(optional_string(&beatmap.audio_filename));
                    ui.end_row();

                    ui.label("MD5");
                    ui.label(optional_string(&beatmap.md5));
                    ui.end_row();

                    ui.label("Beatmap Filename");
                    ui.label(optional_string(&beatmap.beatmap_filename));
                    ui.end_row();

                    ui.label("Ranked Status");
                    ui.label(beatmap.ranked_status.to_string());
                    ui.end_row();

                    ui.label("Hitcircles");
                    ui.label(beatmap.hitcircle_count.to_string());
                    ui.end_row();

                    ui.label("Sliders");
                    ui.label(beatmap.slider_count.to_string());
                    ui.end_row();

                    ui.label("Spinners");
                    ui.label(beatmap.spinner_count.to_string());
                    ui.end_row();

                    ui.label("Last Modified");
                    ui.label(beatmap.last_modification_time.to_string());
                    ui.end_row();

                    ui.label("Approach Rate");
                    ui.label(format!("{:.1}", beatmap.approach_rate));
                    ui.end_row();

                    ui.label("Circle Size");
                    ui.label(format!("{:.1}", beatmap.circle_size));
                    ui.end_row();

                    ui.label("HP Drain");
                    ui.label(format!("{:.1}", beatmap.hp_drain));
                    ui.end_row();

                    ui.label("Overall Difficulty");
                    ui.label(format!("{:.1}", beatmap.overall_difficulty));
                    ui.end_row();

                    ui.label("Slider Velocity");
                    ui.label(format!("{:.2}", beatmap.slider_velocity));
                    ui.end_row();

                    // TODO: Star ratings

                    ui.label("Drain Time");
                    ui.label(format!("{} s", beatmap.drain_time));
                    ui.end_row();

                    ui.label("Total Time");
                    ui.label(format!("{} ms", beatmap.total_time));
                    ui.end_row();

                    ui.label("Audio Preview Time");
                    ui.label(format!("{} ms", beatmap.audio_preview_time));
                    ui.end_row();

                    // TODO: Timing points

                    ui.label("Difficulty ID");
                    ui.label(maybe_signed(beatmap.difficulty_id));
                    ui.end_row();

                    ui.label("Beatmap ID");
                    ui.label(maybe_signed(beatmap.beatmap_id));
                    ui.end_row();

                    ui.label("Thread ID");
                    ui.label(maybe_signed(beatmap.thread_id));
                    ui.end_row();

                    ui.label("Grade (Standard)");
                    ui.label(beatmap.grade_std.to_string());
                    ui.end_row();

                    ui.label("Grade (Taiko)");
                    ui.label(beatmap.grade_taiko.to_string());
                    ui.end_row();

                    ui.label("Grade (Catch)");
                    ui.label(beatmap.grade_catch.to_string());
                    ui.end_row();

                    ui.label("Grade (Mania)");
                    ui.label(beatmap.grade_mania.to_string());
                    ui.end_row();

                    ui.label("Local Offset");
                    ui.label(format!("{} ms", beatmap.local_offset));
                    ui.end_row();

                    ui.label("Stack Leniency");
                    ui.label(format!("{:.1}", beatmap.stack_leniency));
                    ui.end_row();

                    ui.label("Gameplay Mode");
                    ui.label(beatmap.gameplay_mode.to_string());
                    ui.end_row();

                    ui.label("Song Source");
                    ui.label(optional_string(&beatmap.song_source));
                    ui.end_row();

                    ui.label("Song Tags");
                    ui.label(optional_string(&beatmap.song_tags));
                    ui.end_row();

                    ui.label("Online Offset");
                    ui.label(format!("{} ms", beatmap.online_offset));
                    ui.end_row();

                    ui.label("Title Font");
                    ui.label(optional_string(&beatmap.font));
                    ui.end_row();

                    ui.label("Last Played");
                    ui.label(beatmap.last_played.to_string());
                    ui.end_row();

                    ui.label("Is osz2?");
                    ui.label(beatmap.is_osz2.to_string());
                    ui.end_row();

                    ui.label("Folder Name");
                    ui.label(optional_string(&beatmap.folder_name));
                    ui.end_row();

                    ui.label("Last Checked Online");
                    ui.label(beatmap.last_checked_online.to_string());
                    ui.end_row();

                    ui.label("Ignore Hitsounds");
                    ui.label(beatmap.ignore_beatmap_hitsounds.to_string());
                    ui.end_row();

                    ui.label("Ignore Skin");
                    ui.label(beatmap.ignore_beatmap_skin.to_string());
                    ui.end_row();

                    ui.label("Disable Storyboard");
                    ui.label(beatmap.disable_storyboard.to_string());
                    ui.end_row();

                    ui.label("Disable Video");
                    ui.label(beatmap.disable_video.to_string());
                    ui.end_row();

                    ui.label("Visual Override");
                    ui.label(beatmap.visual_override.to_string());
                    ui.end_row();

                    ui.label("Unknown Short");
                    ui.label(optional_string(&beatmap.unknown_u16));
                    ui.end_row();

                    ui.label("Unknown Int");
                    ui.label(beatmap.unknown_u32.to_string());
                    ui.end_row();

                    ui.label("Mania Scroll Speed");
                    ui.label(beatmap.mania_scroll_speed.to_string());
                    ui.end_row();
                })
            })
    });
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
