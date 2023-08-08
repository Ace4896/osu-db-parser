use egui::{Id, RichText};
use osu_db_parser::prelude::*;

use super::{maybe_signed_u32, mods_string, optional_string};

/// A window for displaying beatmap details.
pub struct BeatmapDetailsWindow {
    pub id: Id,
    pub title: String,
    pub visible: bool,
    pub data: BeatmapEntry,
}

impl BeatmapDetailsWindow {
    pub fn view(&mut self, ctx: &egui::Context) {
        egui::Window::new(&self.title)
            .id(self.id)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, true])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        egui::Grid::new(self.id.with("grid")).show(ui, |ui| {
                            ui.label("Size");
                            ui.label(optional_string(&self.data.size));
                            ui.end_row();

                            ui.label("Artist Name");
                            ui.label(optional_string(&self.data.artist_name));
                            ui.end_row();

                            ui.label("Artist Name (Unicode)");
                            ui.label(optional_string(&self.data.artist_name_unicode));
                            ui.end_row();

                            ui.label("Song Title");
                            ui.label(optional_string(&self.data.song_title));
                            ui.end_row();

                            ui.label("Song Title (Unicode)");
                            ui.label(optional_string(&self.data.song_title_unicode));
                            ui.end_row();

                            ui.label("Creator");
                            ui.label(optional_string(&self.data.creator_name));
                            ui.end_row();

                            ui.label("Difficulty");
                            ui.label(optional_string(&self.data.difficulty));
                            ui.end_row();

                            ui.label("Audio Filename");
                            ui.label(optional_string(&self.data.audio_filename));
                            ui.end_row();

                            ui.label("MD5");
                            ui.label(optional_string(&self.data.md5));
                            ui.end_row();

                            ui.label("Beatmap Filename");
                            ui.label(optional_string(&self.data.beatmap_filename));
                            ui.end_row();

                            ui.label("Ranked Status");
                            ui.label(self.data.ranked_status.to_string());
                            ui.end_row();

                            ui.label("Hitcircles");
                            ui.label(self.data.hitcircle_count.to_string());
                            ui.end_row();

                            ui.label("Sliders");
                            ui.label(self.data.slider_count.to_string());
                            ui.end_row();

                            ui.label("Spinners");
                            ui.label(self.data.spinner_count.to_string());
                            ui.end_row();

                            ui.label("Last Modified");
                            ui.label(self.data.last_modification_time.to_string());
                            ui.end_row();

                            ui.label("Approach Rate");
                            ui.label(format!("{:.1}", self.data.approach_rate));
                            ui.end_row();

                            ui.label("Circle Size");
                            ui.label(format!("{:.1}", self.data.circle_size));
                            ui.end_row();

                            ui.label("HP Drain");
                            ui.label(format!("{:.1}", self.data.hp_drain));
                            ui.end_row();

                            ui.label("Overall Difficulty");
                            ui.label(format!("{:.1}", self.data.overall_difficulty));
                            ui.end_row();

                            ui.label("Slider Velocity");
                            ui.label(format!("{:.2}", self.data.slider_velocity));
                            ui.end_row();

                            Self::star_ratings(
                                self.id,
                                ui,
                                "Star Ratings (Standard)",
                                &self.data.star_ratings_std,
                            );
                            ui.end_row();

                            Self::star_ratings(
                                self.id,
                                ui,
                                "Star Ratings (Taiko)",
                                &self.data.star_ratings_taiko,
                            );
                            ui.end_row();

                            Self::star_ratings(
                                self.id,
                                ui,
                                "Star Ratings (Catch)",
                                &self.data.star_ratings_ctb,
                            );
                            ui.end_row();

                            Self::star_ratings(
                                self.id,
                                ui,
                                "Star Ratings (Mania)",
                                &self.data.star_ratings_mania,
                            );
                            ui.end_row();

                            ui.label("Drain Time");
                            ui.label(format!("{} s", self.data.drain_time));
                            ui.end_row();

                            ui.label("Total Time");
                            ui.label(format!("{} ms", self.data.total_time));
                            ui.end_row();

                            ui.label("Audio Preview Time");
                            ui.label(format!("{} ms", self.data.audio_preview_time));
                            ui.end_row();

                            ui.label("Timing Points");

                            if self.data.timing_points.is_empty() {
                                ui.label(RichText::new("N/A").italics());
                            } else {
                                egui::CollapsingHeader::new(format!(
                                    "{} Timing Points",
                                    self.data.timing_points.len()
                                ))
                                .id_source(self.id.with("timing_points"))
                                .show(ui, |ui| {
                                    egui::Grid::new(self.id.with("timing_points_grid")).show(
                                        ui,
                                        |ui| {
                                            ui.label("BPM");
                                            ui.label("Offset / ms");
                                            ui.label("Inherited?");
                                            ui.end_row();

                                            for timing_point in &self.data.timing_points {
                                                ui.label(format!("{:.2}", timing_point.bpm));
                                                ui.label(format!(
                                                    "{:.2}",
                                                    timing_point.song_offset
                                                ));
                                                ui.label(timing_point.inherited.to_string());
                                                ui.end_row();
                                            }
                                        },
                                    );
                                });
                            }

                            ui.end_row();

                            ui.label("Difficulty ID");
                            ui.label(maybe_signed_u32(self.data.difficulty_id));
                            ui.end_row();

                            ui.label("Beatmap ID");
                            ui.label(maybe_signed_u32(self.data.beatmap_id));
                            ui.end_row();

                            ui.label("Thread ID");
                            ui.label(maybe_signed_u32(self.data.thread_id));
                            ui.end_row();

                            ui.label("Grade (Standard)");
                            ui.label(self.data.grade_std.to_string());
                            ui.end_row();

                            ui.label("Grade (Taiko)");
                            ui.label(self.data.grade_taiko.to_string());
                            ui.end_row();

                            ui.label("Grade (Catch)");
                            ui.label(self.data.grade_catch.to_string());
                            ui.end_row();

                            ui.label("Grade (Mania)");
                            ui.label(self.data.grade_mania.to_string());
                            ui.end_row();

                            ui.label("Local Offset");
                            ui.label(format!("{} ms", self.data.local_offset));
                            ui.end_row();

                            ui.label("Stack Leniency");
                            ui.label(format!("{:.1}", self.data.stack_leniency));
                            ui.end_row();

                            ui.label("Gameplay Mode");
                            ui.label(self.data.gameplay_mode.to_string());
                            ui.end_row();

                            ui.label("Song Source");
                            ui.label(optional_string(&self.data.song_source));
                            ui.end_row();

                            ui.label("Song Tags");
                            ui.label(optional_string(&self.data.song_tags));
                            ui.end_row();

                            ui.label("Online Offset");
                            ui.label(format!("{} ms", self.data.online_offset));
                            ui.end_row();

                            ui.label("Title Font");
                            ui.label(optional_string(&self.data.font));
                            ui.end_row();

                            ui.label("Last Played");
                            ui.label(self.data.last_played.to_string());
                            ui.end_row();

                            ui.label("Is osz2?");
                            ui.label(self.data.is_osz2.to_string());
                            ui.end_row();

                            ui.label("Folder Name");
                            ui.label(optional_string(&self.data.folder_name));
                            ui.end_row();

                            ui.label("Last Checked Online");
                            ui.label(self.data.last_checked_online.to_string());
                            ui.end_row();

                            ui.label("Ignore Hitsounds");
                            ui.label(self.data.ignore_beatmap_hitsounds.to_string());
                            ui.end_row();

                            ui.label("Ignore Skin");
                            ui.label(self.data.ignore_beatmap_skin.to_string());
                            ui.end_row();

                            ui.label("Disable Storyboard");
                            ui.label(self.data.disable_storyboard.to_string());
                            ui.end_row();

                            ui.label("Disable Video");
                            ui.label(self.data.disable_video.to_string());
                            ui.end_row();

                            ui.label("Visual Override");
                            ui.label(self.data.visual_override.to_string());
                            ui.end_row();

                            ui.label("Unknown Short");
                            ui.label(optional_string(&self.data.unknown_u16));
                            ui.end_row();

                            ui.label("Unknown Int");
                            ui.label(self.data.unknown_u32.to_string());
                            ui.end_row();

                            ui.label("Mania Scroll Speed");
                            ui.label(self.data.mania_scroll_speed.to_string());
                            ui.end_row();
                        })
                    });
            });
    }

    fn star_ratings(id: Id, ui: &mut egui::Ui, label: &str, ratings: &Option<Vec<StarRating>>) {
        ui.label(label);

        let header_id = id.with(label).with("header");
        let grid_id = id.with(label).with("grid");

        if let Some(ratings) = ratings.as_ref().filter(|r| !r.is_empty()) {
            egui::CollapsingHeader::new(format!("{} Ratings", ratings.len()))
                .id_source(header_id)
                .show(ui, |ui| {
                    egui::Grid::new(grid_id).show(ui, |ui| {
                        for star_rating in ratings {
                            ui.label(mods_string(star_rating.mods));
                            ui.label(format!("{:.02}", star_rating.rating));
                            ui.end_row();
                        }
                    });
                });
        } else {
            ui.label(RichText::new("N/A").italics());
        }
    }
}
