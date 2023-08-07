use egui::Id;
use osu_db_parser::prelude::*;

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
                // TODO: Score/replay details
            });
    }
}
