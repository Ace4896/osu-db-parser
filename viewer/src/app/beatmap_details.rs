use egui::Id;
use osu_db_parser::prelude::*;

use super::{maybe_signed, optional_string};

/// A window for displaying beatmap details.
pub struct BeatmapDetailsWindow {
    pub title: String,
    pub visible: bool,
}

impl BeatmapDetailsWindow {
    pub fn view(&mut self, ctx: &egui::Context, window_id: Id, beatmap: &BeatmapEntry) {
        egui::Window::new(&self.title)
            .id(window_id)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, true])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        egui::Grid::new(window_id.with("grid")).show(ui, |ui| {
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
            });
    }
}
