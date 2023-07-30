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
    bytes::complete::take,
    combinator::{cond, map},
    multi::length_count,
    number::complete::{le_f64, le_u16, le_u32, le_u64},
    IResult,
};
use time::OffsetDateTime;

use crate::{
    common::{
        boolean, gameplay_mode, modifiers, osu_string, windows_datetime, GameplayMode, Mods,
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
    pub lifebar_graph: OsuString,

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

impl ScoreReplay  {
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

/// Parses a score in the `scores.db` file or a `.osr` replay file.
fn score_replay(input: &[u8]) -> IResult<&[u8], ScoreReplay> {
    let (i, gameplay_mode) = gameplay_mode(input)?;
    let (i, version) = le_u32(i)?;
    let (i, beatmap_md5) = osu_string(i)?;
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
    let (i, lifebar_graph) = osu_string(i)?;
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
