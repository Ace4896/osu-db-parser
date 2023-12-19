use egui::Id;
use osu_db_parser::prelude::*;

use super::{maybe_signed_u64, mods_string, optional_string, open_score_in_browser};

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
                ui.add_enabled_ui(self.data.online_score_id != 0, |ui| {
                    if ui.link("View Score Online").clicked() {
                        open_score_in_browser(&self.data);
                    }
                });

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
                    ui.label(mods_string(self.data.mods));
                    ui.end_row();

                    ui.label("Lifebar Graph");

                    if let Some(lifebar) = &self.data.lifebar_graph {
                        use egui::Color32;
                        use egui_plot::{Line, PlotPoints};

                        let plot_points = lifebar
                            .points
                            .iter()
                            .map(|(t, h)| [f64::from(*t), f64::from(*h)])
                            .collect::<PlotPoints>();

                        let line = Line::new(plot_points).color(Color32::WHITE).width(2.0);

                        egui_plot::Plot::new(self.id.with("lifebar_plot"))
                            .allow_drag(false)
                            .allow_scroll(false)
                            .allow_zoom(false)
                            .allow_boxed_zoom(false)
                            .show_axes([false, false])
                            .show_x(false)
                            .show_y(false)
                            .include_x(0.0)
                            .include_y(0.0)
                            .include_y(1.0)
                            .auto_bounds_x()
                            .auto_bounds_y()
                            .show(ui, |plot_ui| plot_ui.line(line));
                    } else {
                        ui.label(egui::RichText::new("N/A").italics());
                    }

                    ui.end_row();

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
