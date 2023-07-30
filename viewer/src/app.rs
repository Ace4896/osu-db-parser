use osu_db_parser::prelude::*;

use crate::widgets::file_dialog::FileDialog;

pub struct MainApp {
    file_dialog: FileDialog,
    waiting_for_db: Option<DatabaseType>,

    opened_dbs: Vec<OpenedDatabase>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseType {
    BeatmapListing,
    CollectionListing,
    ScoreListing,
}

pub enum OpenedDatabase {
    Beatmaps { name: String, db: BeatmapListing },
    Collections { name: String, db: CollectionListing },
    Scores { name: String, db: ScoreListing },
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            file_dialog: FileDialog::default(),
            waiting_for_db: None,

            opened_dbs: Vec::new(),
        }
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::BeatmapListing => write!(f, "Beatmap Listing"),
            DatabaseType::CollectionListing => write!(f, "Collection Listing"),
            DatabaseType::ScoreListing => write!(f, "Score Listing"),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_for_files();
        self.menu_bar(ctx, frame);
    }
}

impl MainApp {
    /// Checks if we are waiting for a file and attempts to parse it if it has been loaded.
    fn check_for_files(&mut self) {
        if let Some(db_type) = self.waiting_for_db {
            if let Some(data) = self.file_dialog.get() {
                let name = format!("#{} - {}", self.opened_dbs.len() + 1, db_type);

                let opened_db = match db_type {
                    DatabaseType::BeatmapListing => BeatmapListing::from_bytes(&data)
                        .map(|db| OpenedDatabase::Beatmaps { name, db }),
                    DatabaseType::CollectionListing => CollectionListing::from_bytes(&data)
                        .map(|db| OpenedDatabase::Collections { name, db }),
                    DatabaseType::ScoreListing => ScoreListing::from_bytes(&data)
                        .map(|db| OpenedDatabase::Scores { name, db }),
                };

                match opened_db {
                    Ok(db) => {
                        log::debug!("Successfully opened {} ({} bytes)", db_type, data.len());
                        self.opened_dbs.push(db);
                    }
                    Err(e) => {
                        log::warn!("Unable to open {}: {}", db_type, e);
                    }
                }
            }
        }
    }

    /// Renders the menu bar in the main window.
    fn menu_bar(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File Menu
                ui.menu_button("File", |ui| {
                    if ui.button("Open osu.db...").clicked() {
                        self.waiting_for_db = Some(DatabaseType::BeatmapListing);
                        self.file_dialog.open();
                    }

                    if ui.button("Open collection.db...").clicked() {
                        self.waiting_for_db = Some(DatabaseType::CollectionListing);
                        self.file_dialog.open();
                    }

                    if ui.button("Open scores.db...").clicked() {
                        self.waiting_for_db = Some(DatabaseType::ScoreListing);
                        self.file_dialog.open();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();

                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    }
                })
            })
        });
    }
}
