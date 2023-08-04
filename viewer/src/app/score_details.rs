use egui::Id;
use osu_db_parser::prelude::*;

/// A window for displaying score details.
pub struct ScoreDetailsWindow {
    pub title: String,
    pub visible: bool,
    pub beatmap_index: usize,
}

impl ScoreDetailsWindow {
    pub fn view(&mut self, ctx: &egui::Context, window_id: Id, score_replay: &ScoreReplay) {
        egui::Window::new(&self.title)
            .id(window_id)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                // TODO: Score details
            });
    }
}
