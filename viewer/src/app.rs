use std::collections::HashMap;

use egui::Id;
use osu_db_parser::prelude::*;

use crate::widgets::file_dialog::FileDialog;

/// Holds the state for the main application.
pub struct MainApp {
    file_dialog: FileDialog,
    pending_file_operation: Option<FileOperation>,

    beatmap_listing: Option<BeatmapListing>,
    displayed_beatmaps: HashMap<usize, WindowDetails<()>>,
    md5_mapping: HashMap<String, usize>,

    collection_listings: Vec<WindowDetails<CollectionListing>>,
    score_listings: Vec<WindowDetails<ScoreListing>>,
    replays: Vec<WindowDetails<ScoreReplay>>,
}

/// Represents generic window details for a value.
pub struct WindowDetails<T> {
    visible: bool,
    title: String,
    data: T,
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

            beatmap_listing: None,
            displayed_beatmaps: HashMap::new(),
            md5_mapping: HashMap::new(),

            collection_listings: Vec::new(),
            score_listings: Vec::new(),
            replays: Vec::new(),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_for_files();

        self.menu_bar(ctx, frame);
        self.displayed_beatmaps(ctx);
        self.collection_listings(ctx);
        self.beatmap_listing(ctx);
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
                                // Update the MD5 mapping with all non-empty hash strings
                                self.md5_mapping = beatmap_listing
                                    .beatmaps
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, b)| {
                                        b.md5
                                            .as_ref()
                                            .filter(|md5| !md5.is_empty())
                                            .map(|md5| (md5.to_string(), i))
                                    })
                                    .collect();

                                // Update the currently displayed values
                                self.beatmap_listing = Some(beatmap_listing);
                                self.displayed_beatmaps.clear();
                            }
                            Err(e) => log::warn!("Unable to open beatmap listing: {}", e),
                        }
                    }
                    FileOperation::GetCollectionListing => {
                        match CollectionListing::from_bytes(&data) {
                            Ok(collection_listing) => {
                                let title = format!(
                                    "Collection Listing #{}",
                                    self.collection_listings.len()
                                );
                                self.collection_listings.push(WindowDetails {
                                    visible: true,
                                    title,
                                    data: collection_listing,
                                });
                            }
                            Err(e) => log::warn!("Unable to open collection listing: {}", e),
                        }
                    }
                    FileOperation::GetScoreListing => match ScoreListing::from_bytes(&data) {
                        Ok(score_listing) => {
                            let title = format!("Score Listing #{}", self.score_listings.len());
                            self.score_listings.push(WindowDetails {
                                visible: true,
                                title,
                                data: score_listing,
                            });
                        }
                        Err(e) => log::warn!("Unable to open score listing: {}", e),
                    },
                    FileOperation::GetReplay => match ScoreReplay::from_bytes(&data) {
                        Ok(replay) => {
                            let title = format!("Replay #{}", self.score_listings.len());
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

                    ui.add_enabled_ui(self.beatmap_listing.is_some(), |ui| {
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
            });
        });
    }

    /// Renders the details for a displayed beatmap in a separate window.
    fn displayed_beatmaps(&mut self, ctx: &egui::Context) {
        let base_id = Id::new("beatmap_details");

        self.displayed_beatmaps
            .retain(|_, window_details| window_details.visible);

        if let Some(BeatmapListing { beatmaps, .. }) = &self.beatmap_listing {
            for (i, window_details) in self.displayed_beatmaps.iter_mut() {
                if let Some(beatmap) = beatmaps.get(*i) {
                    let window_id = base_id.with(i);
                    let grid_id = window_id.with("grid");

                    egui::Window::new(&window_details.title)
                        .id(window_id)
                        .open(&mut window_details.visible)
                        .scroll2([true, true])
                        .show(ctx, |ui| beatmap_details(grid_id, ui, beatmap));
                }
            }
        }
    }

    /// Renders the details for a loaded collection list in a separate window.
    fn collection_listings(&mut self, ctx: &egui::Context) {
        let base_id = Id::new("collection_listing");

        self.collection_listings
            .retain(|window_details| window_details.visible);

        for (listing_index, window_details) in self.collection_listings.iter_mut().enumerate() {
            let window_id = base_id.with(listing_index);

            egui::Window::new(&window_details.title)
                .id(window_id)
                .open(&mut window_details.visible)
                .scroll2([true, true])
                .show(ctx, |ui| {
                    // Version Details
                    ui.horizontal(|ui| {
                        ui.label("Version");
                        ui.label(window_details.data.version.to_string());
                    });

                    // Beatmaps in Collections
                    for (collection_index, collection) in
                        window_details.data.collections.iter().enumerate()
                    {
                        let collection_id = window_id.with(collection_index);

                        egui::CollapsingHeader::new(optional_string(&collection.name))
                            .id_source(collection_id)
                            .show(ui, |ui| {
                                for md5 in collection
                                    .beatmap_md5s
                                    .iter()
                                    .filter_map(|md5| md5.as_ref())
                                {
                                    if let Some((beatmap_index, beatmap)) = self
                                        .md5_mapping
                                        .get(md5)
                                        .map(|i| {
                                            self.beatmap_listing
                                                .as_ref()
                                                .map(|listing| {
                                                    listing.beatmaps.get(*i).map(|b| (i, b))
                                                })
                                                .flatten()
                                        })
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
                                                WindowDetails {
                                                    visible: true,
                                                    title: name,
                                                    data: (),
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
                });
        }
    }

    /// Renders the central panel showing the details of the current beatmap listing.
    fn beatmap_listing(&mut self, ctx: &egui::Context) {
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
                                            WindowDetails {
                                                visible: true,
                                                title: header,
                                                data: (),
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

fn beatmap_details(id: impl Into<Id>, ui: &mut egui::Ui, beatmap: &BeatmapEntry) {
    let id: Id = id.into();

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
