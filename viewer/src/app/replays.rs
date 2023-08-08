use std::collections::HashMap;

use egui::Id;
use osu_db_parser::prelude::*;

use super::score_details::ScoreDetailsWindow;

/// Represents the "Replays" tabbed view.
#[derive(Default)]
pub struct ReplaysView {
    displayed_replays: Vec<ScoreDetailsWindow>,
}

impl ReplaysView {
    /// Loads a replay into this view.
    pub fn load_replay(&mut self, replay: ScoreReplay, beatmaps: &HashMap<String, BeatmapEntry>) {
        let id = Id::new("replay_details").with(self.displayed_replays.len());

        self.displayed_replays.push(ScoreDetailsWindow {
            id,
            title: Self::get_replay_title(&replay, beatmaps),
            visible: true,
            data: replay,
        });
    }

    /// Updates the window titles for each replay that is currently being displayed.
    pub fn update_replay_titles(&mut self, beatmaps: &HashMap<String, BeatmapEntry>) {
        for displayed_replay in self.displayed_replays.iter_mut() {
            displayed_replay.title = Self::get_replay_title(&displayed_replay.data, beatmaps);
        }
    }

    /// Renders the view for each replay that is currently loaded.
    pub fn view(&mut self, ctx: &egui::Context) {
        // Unload any replays whose window is closed
        self.displayed_replays.retain(|w| w.visible);

        // Display the remaining windows
        for replay_window in self.displayed_replays.iter_mut() {
            replay_window.view(ctx);
        }

        // Empty Central Panel
        egui::CentralPanel::default().show(ctx, |_| {});
    }

    /// Gets the title for a particular replay.
    fn get_replay_title(replay: &ScoreReplay, beatmaps: &HashMap<String, BeatmapEntry>) -> String {
        if let Some(beatmap) = replay
            .beatmap_md5
            .as_ref()
            .and_then(|md5| beatmaps.get(md5))
        {
            format!(
                "{} - {} - {} [{}]",
                replay.player_name.clone().unwrap_or_default(),
                beatmap.artist_name.clone().unwrap_or_default(),
                beatmap.song_title.clone().unwrap_or_default(),
                beatmap.difficulty.clone().unwrap_or_default(),
            )
        } else {
            format!(
                "{} - Unknown Beatmap (MD5: {})",
                replay.player_name.clone().unwrap_or_default(),
                replay.beatmap_md5.clone().unwrap_or_default(),
            )
        }
    }
}
