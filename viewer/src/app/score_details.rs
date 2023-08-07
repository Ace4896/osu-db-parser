use egui::Id;
use osu_db_parser::prelude::*;

use super::{maybe_signed_u64, optional_string};

/// A window for displaying score details.
pub struct ScoreDetailsWindow {
    pub id: Id,
    pub title: String,
    pub visible: bool,
    pub data: ScoreReplay,
}

impl ScoreDetailsWindow {
    /// Renders this window to display score/replay details.
    pub fn view(&mut self, ctx: &egui::Context) {
        egui::Window::new(&self.title)
            .id(self.id)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                egui::Grid::new(self.id.with("grid")).show(ui, |ui| {
                    ui.label("Gameplay Mode");
                    ui.label(self.data.gameplay_mode.to_string());
                    ui.end_row();

                    ui.label("Version");
                    ui.label(self.data.version.to_string());
                    ui.end_row();

                    ui.label("Beatmap MD5");
                    ui.label(optional_string(&self.data.beatmap_md5));
                    ui.end_row();

                    ui.label("Replay MD5");
                    ui.label(optional_string(&self.data.replay_md5));
                    ui.end_row();

                    ui.label("Player Name");
                    ui.label(self.data.player_name.clone().unwrap_or_default());
                    ui.end_row();

                    ui.label("300s");
                    ui.label(self.data.hits_300.to_string());
                    ui.end_row();

                    ui.label("100s");
                    ui.label(self.data.hits_100.to_string());
                    ui.end_row();

                    ui.label("50s");
                    ui.label(self.data.hits_50.to_string());
                    ui.end_row();

                    ui.label("Gekis");
                    ui.label(self.data.hits_geki.to_string());
                    ui.end_row();

                    ui.label("Katus");
                    ui.label(self.data.hits_katu.to_string());
                    ui.end_row();

                    ui.label("Misses");
                    ui.label(self.data.misses.to_string());
                    ui.end_row();

                    ui.label("Score");
                    ui.label(self.data.score.to_string());
                    ui.end_row();

                    ui.label("Max Combo");
                    ui.label(self.data.max_combo.to_string());
                    ui.end_row();

                    ui.label("Is Perfect Combo");
                    ui.label(self.data.is_perfect_combo.to_string());
                    ui.end_row();

                    ui.label("Mods");
                    ui.label(format!("{:?}", self.data.mods));
                    ui.end_row();

                    // TODO: Lifebar graph - needs a dedicated renderer
                    // It's stored as comma-separated key-value pairs:
                    // - Key is the timestamp in milliseconds
                    // - Value is the life value - range is [0, 1], where 1 indicates full health
                    //
                    // Probably need something similar to egui's plotting, most likely need to adjust the parsed model

                    ui.label("Timestamp");
                    ui.label(self.data.timestamp.to_string());
                    ui.end_row();

                    ui.label("Has Replay Data");
                    ui.label(
                        self.data
                            .replay_data
                            .as_ref()
                            .is_some_and(|data| !data.is_empty())
                            .to_string(),
                    );
                    ui.end_row();

                    ui.label("Online Score ID");
                    ui.label(maybe_signed_u64(self.data.online_score_id));
                    ui.end_row();

                    ui.label("Additional Mod Information");
                    ui.label(optional_string(&self.data.additional_mod_info));
                    ui.end_row();
                });
            });
    }
}
