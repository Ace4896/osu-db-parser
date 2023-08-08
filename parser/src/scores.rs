//! Models for the `scores.db` database file, which contains information on locally achieved scores.
//!
//! From the [osu! wiki], the score data is identical to the [replay format], but has no input-related data.
//! So the functions provided here can also be used to parse `.osr` replay files.
//!
//! [osu! wiki]: https://github.com/ppy/osu/wiki/Legacy-database-file-structure#scoresdb
//! [replay format]: https://osu.ppy.sh/wiki/en/Client/File_formats/osr_%28file_format%29

use std::path::Path;

use flagset::FlagSet;
use nom::{
    bytes::complete::{tag, take},
    character::complete::digit1,
    combinator::{cond, map, map_res},
    multi::{length_count, many0},
    number::complete::{float, le_f64, le_u16, le_u32, le_u64},
    sequence::{separated_pair, terminated},
    IResult,
};
use time::OffsetDateTime;

use crate::{
    common::{
        boolean, gameplay_mode, modifiers, osu_string, windows_datetime, GameplayMode, Grade, Mods,
        OsuString,
    },
    error::Error,
};

/// Represents the `scores.db` file.
#[derive(Clone, Debug)]
pub struct ScoreListing {
    /// Version (e.g. 20150204)
    pub version: u32,

    /// List of scores achieved per beatmap
    pub beatmap_scores: Vec<BeatmapScores>,
}

/// Represents a list of scores for a beatmap in the `scores.db` file.
#[derive(Clone, Debug)]
pub struct BeatmapScores {
    /// Beatmap MD5 hash
    pub md5: OsuString,

    /// Scores achieved for this beatmap
    pub scores: Vec<ScoreReplay>,
}

/// Represents an individual replay for a score on a beatmap, either in the `scores.db` file or a `.osr` replay.
///
/// Note that the compressed replay data may not be present, e.g. if this came from the `scores.db` file.
#[derive(Clone, Debug)]
pub struct ScoreReplay {
    /// osu! gameplay mode
    pub gameplay_mode: GameplayMode,

    /// Version of this score/replay (e.g. 20150203)
    pub version: u32,

    /// Beatmap MD5 hash
    pub beatmap_md5: OsuString,

    /// Player name
    pub player_name: OsuString,

    /// Replay MD5 hash
    pub replay_md5: OsuString,

    /// Number of 300's
    pub hits_300: u16,

    /// Number of 100's in osu!, 150's in osu!taiko, 100's in osu!catch, 100's in osu!mania
    pub hits_100: u16,

    /// Number of 50's in osu!, small fruit in osu!catch, 50's in osu!mania
    pub hits_50: u16,

    /// Number of Gekis in osu!, Max 300's in osu!mania
    pub hits_geki: u16,

    /// Number of Katus in osu!, 200's in osu!mania
    pub hits_katu: u16,

    /// Number of misses
    pub misses: u16,

    /// Replay score
    pub score: u32,

    /// Max combo
    pub max_combo: u16,

    /// Perfect combo
    pub is_perfect_combo: bool,

    /// Mods used
    pub mods: FlagSet<Mods>,

    /// Life bar graph (see [replay format details](https://osu.ppy.sh/wiki/en/Client/File_formats/osr_%28file_format%29#format)).
    /// Only present when parsing a `.osr` replay file.
    pub lifebar_graph: Option<LifebarGraph>,

    /// Timestamp of replay
    pub timestamp: OffsetDateTime,

    /// LZMA Compressed replay data. Only present when parsing a `.osr` replay file.
    pub replay_data: Option<Vec<u8>>,

    /// Online Score ID
    pub online_score_id: u64,

    /// Additional mod information; only present if Target Practice is enabled.
    ///
    /// When target practice is enabled, this is the total accuracy of all hits.
    /// Divide this by the number of targets in the map to find the accuracy displayed in-game.
    pub additional_mod_info: Option<f64>,
}

/// Represents the lifebar graph in a .osr replay file.
#[derive(Clone, Debug, PartialEq)]
pub struct LifebarGraph {
    pub points: Vec<(u32, f32)>,
}

impl std::fmt::Display for LifebarGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Each point is represented as a 'time|hp' pair, where each pair is comma separated
        // There should also be a trailing comma at the end of the string
        write!(
            f,
            "{}",
            self.points
                .iter()
                .map(|(t, h)| format!("{}|{},", t, h))
                .collect::<String>()
        )
    }
}

impl ScoreListing {
    /// Parses the contents of a `collection.db` file.
    pub fn from_bytes(data: &[u8]) -> Result<ScoreListing, Error> {
        let (_, listing) = score_listing(data).map_err(|e| e.to_owned())?;
        Ok(listing)
    }

    /// Convenience method for reading the contents of an `collection.db` file and parsing it as a `ScoreListing`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ScoreListing, Error> {
        let data = std::fs::read(path)?;
        Self::from_bytes(&data)
    }
}

impl ScoreReplay {
    /// Parses the contents of a `.osr` replay.
    pub fn from_bytes(data: &[u8]) -> Result<ScoreReplay, Error> {
        let (_, listing) = score_replay(data).map_err(|e| e.to_owned())?;
        Ok(listing)
    }

    /// Convenience method for reading the contents of an `collection.db` file and parsing it as a `ScoreListing`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ScoreReplay, Error> {
        let data = std::fs::read(path)?;
        Self::from_bytes(&data)
    }

    /// Calculates the accuracy percentage for this score/replay, using the formulae from the [osu! wiki](https://osu.ppy.sh/wiki/en/Gameplay/Accuracy).
    pub fn accuracy(&self) -> f64 {
        let accuracy = match self.gameplay_mode {
            GameplayMode::Standard => {
                (300.0 * self.hits_300 as f64
                    + 100.0 * self.hits_100 as f64
                    + 50.0 * self.hits_50 as f64)
                    / (300.0 * (self.hits_300 + self.hits_100 + self.hits_50 + self.misses) as f64)
            }

            // Taiko only has Great/Good/Miss; hits_50 in the replay data isn't used
            GameplayMode::Taiko => {
                (self.hits_300 as f64 + 0.5 * self.hits_100 as f64)
                    / (self.hits_300 + self.hits_100 + self.misses) as f64
            }

            // For Catch:
            // - 300 = Caught Fruit
            // - 100 = Caught Drops
            // - 50 = Caught Droplets
            // - Miss = Missed Fruits + Drops
            // - Katu = Missed Droplets
            GameplayMode::Catch => {
                (self.hits_300 + self.hits_100 + self.hits_50) as f64
                    / (self.hits_300 + self.hits_100 + self.hits_50 + self.misses + self.hits_katu)
                        as f64
            }

            // For Mania:
            // - Geki = Rainbow 300
            // - Katu = 200
            GameplayMode::Mania => {
                let hits_300_or_below = 300.0 * self.hits_300 as f64
                    + 200.0 * self.hits_katu as f64
                    + 100.0 * self.hits_100 as f64
                    + 50.0 * self.hits_50 as f64;
                let total =
                    (self.hits_geki + self.hits_300 + self.hits_100 + self.hits_50 + self.misses)
                        as f64;

                // Rainbow 300s have different weighting for ScoreV1/2
                // ScoreV1 uses 300, ScoreV2 uses 305
                if self.mods.contains(Mods::ScoreV2) {
                    (300.0 * self.hits_geki as f64 + hits_300_or_below) / (300.0 * total)
                } else {
                    (305.0 * self.hits_geki as f64 + hits_300_or_below) / (305.0 * total)
                }
            }
        };

        accuracy * 100.0
    }

    /// Determines the grade achieved for this replay, using the calculations from the [osu! wiki](https://osu.ppy.sh/wiki/en/Gameplay/Grade).
    pub fn grade(&self) -> Grade {
        // Determine the initial grade (before modifiers)
        let initial_grade = match self.gameplay_mode {
            // Standard:
            // - SS = 100% accuracy
            // - S  = Over 90% 300s, at most 1% 50s, and no misses
            // - A  = Over 80% 300s and no misses OR over 90% 300s
            // - B  = Over 70% 300s and no misses OR over 80% 300s
            // - C  = Over 60% 300s
            // - D  = Anything else
            GameplayMode::Standard => {
                if self.hits_300 > 0 && self.hits_100 == 0 && self.hits_50 == 0 && self.misses == 0
                {
                    return Grade::SS;
                }

                let total_objects = self.hits_300 as f64
                    + self.hits_100 as f64
                    + self.hits_50 as f64
                    + self.misses as f64;

                let ratio_300 = self.hits_300 as f64 / total_objects;
                let ratio_50 = self.hits_50 as f64 / total_objects;

                match (ratio_300, ratio_50, self.misses) {
                    (r_300, r_50, 0) if r_300 >= 0.9 && r_50 <= 0.01 => Grade::S,
                    (r_300, _, m) if (r_300 >= 0.8 && m == 0) || (r_300 >= 0.9) => Grade::A,
                    (r_300, _, m) if (r_300 >= 0.7 && m == 0) || (r_300 >= 0.8) => Grade::B,
                    (r_300, _, _) if r_300 >= 0.6 => Grade::C,
                    _ => Grade::D,
                }
            }

            // Taiko:
            // - SS = 100% accuracy
            // - S  = Over 90% GREATs and no misses
            // - A  = Over 80% GREATs and no misses OR over 90% GREATs
            // - B  = Over 70% GREATs and no misses OR over 80% GREATs
            // - C  = Over 60% GREATs
            // - D  = Any other passing score
            GameplayMode::Taiko => {
                if self.hits_300 > 0 && self.hits_100 == 0 && self.misses == 0 {
                    return Grade::SS;
                }

                let total_objects =
                    self.hits_300 as f64 + self.hits_100 as f64 + self.misses as f64;

                let ratio_great = self.hits_300 as f64 / total_objects;

                match (ratio_great, self.misses) {
                    (r_great, m) if r_great >= 0.9 && m == 0 => Grade::S,
                    (r_great, m) if (r_great >= 0.8 && m == 0) || r_great >= 0.9 => Grade::A,
                    (r_great, m) if (r_great >= 0.7 && m == 0) || r_great >= 0.8 => Grade::B,
                    (r_great, _) if r_great >= 0.6 => Grade::C,
                    _ => Grade::D,
                }
            }

            // Catch:
            // - SS = 100% accuracy
            // - S  = 98.01% to 99.99% accuracy (an S rank is possible even with several misses, like in osu!mania)
            // - A  = 94.01% to 98.00% accuracy
            // - B  = 90.01% to 94.00% accuracy
            // - C  = 85.01% to 90.00% accuracy
            // - D  = Any other accuracy under 85.00%
            GameplayMode::Catch => {
                // If there are no missed fruits, drops or droplets, then should be an SS
                if self.misses == 0 && self.hits_katu == 0 {
                    return Grade::SS;
                }

                match self.accuracy() {
                    a if a > 98.0 => Grade::S,
                    a if a > 94.0 => Grade::A,
                    a if a > 90.0 => Grade::B,
                    a if a > 85.0 => Grade::C,
                    _ => Grade::D,
                }
            }

            // Mania:
            // - SS = 100% accuracy
            // - S  = Over 95% accuracy (an S rank is possible even with several misses, like in osu!catch)
            // - A  = Over 90% accuracy
            // - B  = Over 80% accuracy
            // - C  = Over 70% accuracy
            // - D  = Anything else
            GameplayMode::Mania => {
                // Accuracy differs between ScoreV1 and ScoreV2
                // In ScoreV1, scores with only rainbow 300s + 300s are considered SS
                // In ScoreV2, scores with only rainbow 300s are considered SS
                if self.hits_100 == 0
                    && self.hits_50 == 0
                    && self.hits_katu == 0
                    && self.misses == 0
                    && ((self.mods.contains(Mods::ScoreV2)
                        && self.hits_geki > 0
                        && self.hits_300 == 0)
                        || (!self.mods.contains(Mods::ScoreV2) && self.hits_geki > 0
                            || self.hits_300 > 0))
                {
                    return Grade::SS;
                }

                match self.accuracy() {
                    a if a >= 95.0 => Grade::S,
                    a if a >= 90.0 => Grade::A,
                    a if a >= 80.0 => Grade::B,
                    a if a >= 70.0 => Grade::C,
                    _ => Grade::D,
                }
            }
        };

        // See if this needs to be converted to a silver SS or silver S rank
        // This is needed when Hidden, Flashlight or Fade In are present
        let contains_silver_mod = self.mods.contains(Mods::Hidden)
            || self.mods.contains(Mods::Flashlight)
            || self.mods.contains(Mods::FadeIn);

        match (initial_grade, contains_silver_mod) {
            (Grade::SS, true) => Grade::SilverSS,
            (Grade::S, true) => Grade::SilverS,
            (g, _) => g,
        }
    }
}

/// Parses a `scores.db` file.
fn score_listing(input: &[u8]) -> IResult<&[u8], ScoreListing> {
    let (i, version) = le_u32(input)?;
    let (i, beatmap_scores) = length_count(le_u32, beatmap_scores)(i)?;

    Ok((
        i,
        ScoreListing {
            version,
            beatmap_scores,
        },
    ))
}

/// Parses the scores for a particular beatmap in the `scores.db` file.
fn beatmap_scores(input: &[u8]) -> IResult<&[u8], BeatmapScores> {
    let (i, md5) = osu_string(input)?;
    let (i, scores) = length_count(le_u32, score_replay)(i)?;

    Ok((i, BeatmapScores { md5, scores }))
}

fn lifebar_graph(input: &[u8]) -> IResult<&[u8], Option<LifebarGraph>> {
    // The lifebar graph is stored as a string, so parse this first
    let (i, lifebar) = osu_string(input)?;

    if let Some(lifebar) = lifebar {
        // Then, parse the string values
        let points = lifebar_graph_points(&lifebar)
            .map(|(_, p)| p)
            .map_err(|e| e.map_input(|_| i))?;

        Ok((i, Some(LifebarGraph { points })))
    } else {
        Ok((i, None))
    }
}

/// Parses the 'time|hp' points within a lifebar graph string.
fn lifebar_graph_points(input: &str) -> IResult<&str, Vec<(u32, f32)>> {
    many0(terminated(
        separated_pair(map_res(digit1, |s: &str| s.parse::<u32>()), tag("|"), float),
        tag(","),
    ))(input)
}

/// Parses a score in the `scores.db` file or a `.osr` replay file.
fn score_replay(input: &[u8]) -> IResult<&[u8], ScoreReplay> {
    let (i, gameplay_mode) = gameplay_mode(input)?;
    let (i, version) = le_u32(i)?;
    let (i, beatmap_md5) = osu_string(i)?;
    let (i, player_name) = osu_string(i)?;
    let (i, replay_md5) = osu_string(i)?;
    let (i, hits_300) = le_u16(i)?;
    let (i, hits_100) = le_u16(i)?;
    let (i, hits_50) = le_u16(i)?;
    let (i, hits_geki) = le_u16(i)?;
    let (i, hits_katu) = le_u16(i)?;
    let (i, misses) = le_u16(i)?;

    let (i, score) = le_u32(i)?;
    let (i, max_combo) = le_u16(i)?;
    let (i, is_perfect_combo) = boolean(i)?;
    let (i, mods) = modifiers(i)?;
    let (i, lifebar_graph) = lifebar_graph(i)?;
    let (i, timestamp) = windows_datetime(i)?;

    // If replay data length is 0xFFFFFFFF (-1), then no replay data is present (e.g. comes from scores.db)
    let (i, replay_data_length) = le_u32(i)?;
    let (i, replay_data) = cond(
        replay_data_length != 0xFFFFFFFF,
        map(take(replay_data_length as usize), |d: &[u8]| d.to_vec()),
    )(i)?;

    let (i, online_score_id) = le_u64(i)?;

    // At the moment, additional mod information is only present when target practice is enabled
    let (i, additional_mod_info) = cond(mods.contains(Mods::TargetPractice), le_f64)(i)?;

    Ok((
        i,
        ScoreReplay {
            gameplay_mode,
            version,
            beatmap_md5,
            player_name,
            replay_md5,
            hits_300,
            hits_100,
            hits_50,
            hits_geki,
            hits_katu,
            misses,
            score,
            max_combo,
            is_perfect_combo,
            mods,
            lifebar_graph,
            timestamp,
            replay_data,
            online_score_id,
            additional_mod_info,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifebar_graph_parses_correctly() {
        let empty_bytes = vec![0x00];
        let zero_bytes = vec![0x0b, 0x00];

        let non_empty = "1676|1,3732|1,5805|1,7847|1,9909|1,";
        let mut non_empty_bytes = vec![0x0b, non_empty.len() as u8];
        non_empty_bytes.extend_from_slice(non_empty.as_bytes());

        // Sanity check to ensure that string is formatted correctly
        assert_eq!(
            Ok((
                &[][..],
                Some("1676|1,3732|1,5805|1,7847|1,9909|1,".to_string())
            )),
            osu_string(&non_empty_bytes)
        );

        // Parsing the empty and zero-length strings
        assert_eq!(Ok((&[][..], None)), lifebar_graph(&empty_bytes));
        assert_eq!(
            Ok((&[][..], Some(LifebarGraph { points: Vec::new() }))),
            lifebar_graph(&zero_bytes)
        );

        // Parsing the non-empty string
        assert_eq!(
            Ok((
                &[][..],
                Some(LifebarGraph {
                    points: vec![
                        (1676, 1.0),
                        (3732, 1.0),
                        (5805, 1.0),
                        (7847, 1.0),
                        (9909, 1.0),
                    ],
                })
            )),
            lifebar_graph(&non_empty_bytes)
        );
    }

    #[test]
    fn lifebar_graph_display_is_correct() {
        let graph = LifebarGraph {
            points: vec![
                (1676, 1.0),
                (3732, 1.0),
                (5805, 1.0),
                (7847, 1.0),
                (9909, 1.0),
            ],
        };

        assert_eq!("1676|1,3732|1,5805|1,7847|1,9909|1,", graph.to_string());
    }
}
