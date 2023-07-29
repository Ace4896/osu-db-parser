//! Models for the main `osu.db` database file, which contains information on installed beatmaps.

use flagset::FlagSet;
use nom::{
    bytes::complete::tag,
    combinator::{cond, map},
    multi::length_count,
    number::complete::{le_f32, le_f64, le_u16, le_u32, u8},
    sequence::{preceded, tuple},
    IResult,
};
use time::OffsetDateTime;

use crate::common::{
    boolean, gameplay_mode, osu_string, windows_datetime, GameplayMode, Mods, OsuString,
};

// TODO: A couple of fields could be represented with more meaningful structs/enums

/// Represents the `osu.db` file.
#[derive(Clone, Debug)]
pub struct BeatmapListing {
    /// osu! version (e.g. 20150203)
    pub version: u32,

    /// Folder count
    pub folder_count: u32,

    /// AccountUnlocked (only false when the account is locked or banned in any way)
    pub account_unlocked: bool,

    /// Date the account will be unlocked
    pub account_unlock_date: OffsetDateTime,

    /// Player name
    pub player_name: OsuString,

    /// Beatmaps
    pub beatmaps: Vec<BeatmapEntry>,

    /// User permissions
    pub user_permissions: u32,
}

/// Represents a beatmap entry found in `osu.db`.
#[derive(Clone, Debug)]
pub struct BeatmapEntry {
    /// Size in bytes of the beatmap entry. Only present if version is less than 20191106.
    pub size: Option<u32>,

    /// Artist name
    pub artist_name: OsuString,

    /// Artist name, in Unicode
    pub artist_name_unicode: OsuString,

    /// Song title
    pub song_title: OsuString,

    /// Song title, in Unicode
    pub song_title_unicode: OsuString,

    /// Creator name
    pub creator_name: OsuString,

    /// Difficulty (e.g. Hard, Insane, etc.)
    pub difficulty: OsuString,

    /// Audio file name
    pub audio_filename: OsuString,

    /// MD5 hash of the beatmap
    pub md5: OsuString,

    /// Name of the .osu file corresponding to this beatmap
    pub beatmap_filename: OsuString,

    /// Ranked status (0 = unknown, 1 = unsubmitted, 2 = pending/wip/graveyard, 3 = unused, 4 = ranked, 5 = approved, 6 = qualified, 7 = loved)
    pub ranked_status: RankedStatus,

    /// Number of hitcircles
    pub hitcircle_count: u16,

    /// Number of sliders (note: this will be present in every mode)
    pub slider_count: u16,

    /// Number of spinners (note: this will be present in every mode)
    pub spinner_count: u16,

    /// Last modification time, Windows ticks
    pub last_modification_time: OffsetDateTime,

    /// Approach rate. Byte if the version is less than 20140609, Single otherwise.
    pub approach_rate: f32,

    /// Circle size. Byte if the version is less than 20140609, Single otherwise.
    pub circle_size: f32,

    /// HP drain. Byte if the version is less than 20140609, Single otherwise.
    pub hp_drain: f32,

    /// Overall difficulty. Byte if the version is less than 20140609, Single otherwise.
    pub overall_difficulty: f32,

    /// Slider velocity
    pub slider_velocity: f64,

    /// Star Rating info for osu! standard
    pub star_ratings_std: Vec<(FlagSet<Mods>, f64)>,

    /// Star Rating info for Taiko
    pub star_ratings_taiko: Vec<(FlagSet<Mods>, f64)>,

    /// Star Rating info for CTB
    pub star_ratings_ctb: Vec<(FlagSet<Mods>, f64)>,

    /// Star Rating info for osu!mania
    pub star_ratings_mania: Vec<(FlagSet<Mods>, f64)>,

    /// Drain time, in seconds
    pub drain_time: u32,

    /// Total time, in milliseconds
    pub total_time: u32,

    /// Time when the audio preview when hovering over a beatmap in beatmap select starts, in milliseconds
    pub audio_preview_time: u32,

    /// Timing points
    pub timing_points: Vec<TimingPoint>,

    /// Difficulty ID
    pub difficulty_id: u32,

    /// Beatmap ID
    pub beatmap_id: u32,

    /// Thread ID
    pub thread_id: u32,

    /// Grade achieved in osu! standard
    pub grade_std: u8,

    /// Grade achieved in taiko
    pub grade_taiko: u8,

    /// Grade achieved in CTB
    pub grade_catch: u8,

    /// Grade achieved in osu!mania
    pub grade_mania: u8,

    /// Local beatmap offset
    pub local_offset: u16,

    /// Stack leniency
    pub stack_leniency: f32,

    /// osu! gameplay mode
    pub gameplay_mode: GameplayMode,

    /// Song source
    pub song_source: OsuString,

    /// Song tags
    pub song_tags: OsuString,

    /// Online offset
    pub online_offset: u16,

    /// Font used for the title of the song
    pub font: OsuString,

    /// Is beatmap unplayed
    pub is_unplayed: bool,

    /// Last time when beatmap was played
    pub last_played: OffsetDateTime,

    /// Is the beatmap osz2
    pub is_osz2: bool,

    /// Folder name of the beatmap, relative to Songs folder
    pub folder_name: OsuString,

    /// Last time when beatmap was checked against osu! repository
    pub last_checked_online: OffsetDateTime,

    /// Ignore beatmap sound
    pub ignore_beatmap_hitsounds: bool,

    /// Ignore beatmap skin
    pub ignore_beatmap_skin: bool,

    /// Disable storyboard
    pub disable_storyboard: bool,

    /// Disable video
    pub disable_video: bool,

    /// Mania scroll speed
    pub mania_scroll_speed: u8,
}

/// Represents the ranked status of a beatmap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RankedStatus {
    Unknown = 0,
    Unsubmitted = 1,

    /// Pending / WIP / Graveyard
    Pending = 2,

    // NOTE: 3 is unused
    Ranked = 4,
    Approved = 5,
    Qualified = 6,
    Loved = 7,
}

/// Represents a timing point found in `osu.db`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimingPoint {
    /// The BPM of this timing point.
    pub bpm: f64,

    /// The offset into the song.
    pub song_offset: f64,

    /// Whether this timing point is inherited.
    pub inherited: bool,
}

/// Parses an `osu.db` file.
fn beatmap_listing(input: &[u8]) -> IResult<&[u8], BeatmapListing> {
    let (i, version) = le_u32(input)?;
    let (i, folder_count) = le_u32(i)?;
    let (i, account_unlocked) = boolean(i)?;
    let (i, account_unlock_date) = windows_datetime(i)?;
    let (i, player_name) = osu_string(i)?;
    let (i, beatmaps) = length_count(le_u32, beatmap_entry(version))(i)?;
    let (i, user_permissions) = le_u32(i)?;

    Ok((
        i,
        BeatmapListing {
            version,
            folder_count,
            account_unlocked,
            account_unlock_date,
            player_name,
            beatmaps,
            user_permissions,
        },
    ))
}

/// Parses a beatmap entry in an `osu.db` file.
fn beatmap_entry(version: u32) -> impl Fn(&[u8]) -> IResult<&[u8], BeatmapEntry> {
    let parse_difficulty: fn(&[u8]) -> IResult<&[u8], f32> = if version < 20140609 {
        |i: &[u8]| map(u8, |b| b as f32)(i)
    } else {
        |i: &[u8]| le_f32(i)
    };

    move |input| {
        let (i, size) = cond(version < 20191106, le_u32)(input)?;
        let (i, artist_name) = osu_string(i)?;
        let (i, artist_name_unicode) = osu_string(i)?;
        let (i, song_title) = osu_string(i)?;
        let (i, song_title_unicode) = osu_string(i)?;
        let (i, creator_name) = osu_string(i)?;
        let (i, difficulty) = osu_string(i)?;
        let (i, audio_filename) = osu_string(i)?;
        let (i, md5) = osu_string(i)?;
        let (i, beatmap_filename) = osu_string(i)?;

        let (i, ranked_status) = ranked_status(i)?;
        let (i, hitcircle_count) = le_u16(i)?;
        let (i, slider_count) = le_u16(i)?;
        let (i, spinner_count) = le_u16(i)?;
        let (i, last_modification_time) = windows_datetime(i)?;
        let (i, approach_rate) = parse_difficulty(i)?;
        let (i, circle_size) = parse_difficulty(i)?;
        let (i, hp_drain) = parse_difficulty(i)?;
        let (i, overall_difficulty) = parse_difficulty(i)?;
        let (i, slider_velocity) = le_f64(i)?;

        let (i, star_ratings_std) = star_ratings(i)?;
        let (i, star_ratings_taiko) = star_ratings(i)?;
        let (i, star_ratings_ctb) = star_ratings(i)?;
        let (i, star_ratings_mania) = star_ratings(i)?;
        let (i, drain_time) = le_u32(i)?;
        let (i, total_time) = le_u32(i)?;
        let (i, audio_preview_time) = le_u32(i)?;
        let (i, timing_points) = length_count(le_u32, timing_point)(i)?;
        let (i, difficulty_id) = le_u32(i)?;
        let (i, beatmap_id) = le_u32(i)?;

        let (i, thread_id) = le_u32(i)?;
        let (i, grade_std) = u8(i)?;
        let (i, grade_taiko) = u8(i)?;
        let (i, grade_catch) = u8(i)?;
        let (i, grade_mania) = u8(i)?;
        let (i, local_offset) = le_u16(i)?;
        let (i, stack_leniency) = le_f32(i)?;
        let (i, gameplay_mode) = gameplay_mode(i)?;
        let (i, song_source) = osu_string(i)?;
        let (i, song_tags) = osu_string(i)?;

        let (i, online_offset) = le_u16(i)?;
        let (i, font) = osu_string(i)?;
        let (i, is_unplayed) = boolean(i)?;
        let (i, last_played) = windows_datetime(i)?;
        let (i, is_osz2) = boolean(i)?;
        let (i, folder_name) = osu_string(i)?;
        let (i, last_checked_online) = windows_datetime(i)?;
        let (i, ignore_beatmap_hitsounds) = boolean(i)?;
        let (i, ignore_beatmap_skin) = boolean(i)?;
        let (i, disable_storyboard) = boolean(i)?;

        let (i, disable_video) = boolean(i)?;

        // NOTE: Unused f32 optional field, only present if version is less than 20140609
        let (i, _) = cond(version < 20140609, le_f32)(i)?;

        // NOTE: Unused u32 field (appears to be last modification time as well)
        let (i, _) = le_u32(i)?;

        let (i, mania_scroll_speed) = u8(i)?;

        Ok((
            i,
            BeatmapEntry {
                size,
                artist_name,
                artist_name_unicode,
                song_title,
                song_title_unicode,
                creator_name,
                difficulty,
                audio_filename,
                md5,
                beatmap_filename,
                ranked_status,
                hitcircle_count,
                slider_count,
                spinner_count,
                last_modification_time,
                approach_rate,
                circle_size,
                hp_drain,
                overall_difficulty,
                slider_velocity,
                star_ratings_std,
                star_ratings_taiko,
                star_ratings_ctb,
                star_ratings_mania,
                drain_time,
                total_time,
                audio_preview_time,
                timing_points,
                difficulty_id,
                beatmap_id,
                thread_id,
                grade_std,
                grade_taiko,
                grade_catch,
                grade_mania,
                local_offset,
                stack_leniency,
                gameplay_mode,
                song_source,
                song_tags,
                online_offset,
                font,
                is_unplayed,
                last_played,
                is_osz2,
                folder_name,
                last_checked_online,
                ignore_beatmap_hitsounds,
                ignore_beatmap_skin,
                disable_storyboard,
                disable_video,
                mania_scroll_speed,
            },
        ))
    }
}

/// Parses a ranked status value.
fn ranked_status(input: &[u8]) -> IResult<&[u8], RankedStatus> {
    use RankedStatus::*;

    let (i, status) = u8(input)?;
    let status = match status {
        0 => Unknown,
        1 => Unsubmitted,
        2 => Pending,
        4 => Ranked,
        5 => Approved,
        6 => Qualified,
        7 => Loved,
        _ => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Switch,
            }))
        }
    };

    Ok((i, status))
}

/// Parses a integer-double pair found in `osu.db`.
fn int_double_pair(input: &[u8]) -> IResult<&[u8], (u32, f64)> {
    let (i, int) = preceded(tag(&[0x08]), le_u32)(input)?;
    let (i, double) = preceded(tag(&[0x0d]), le_f64)(i)?;

    Ok((i, (int, double)))
}

/// Parses a timing point found in `osu.db`.
fn timing_point(input: &[u8]) -> IResult<&[u8], TimingPoint> {
    map(
        tuple((le_f64, le_f64, boolean)),
        |(bpm, song_offset, inherited)| TimingPoint {
            bpm,
            song_offset,
            inherited,
        },
    )(input)
}

/// Parses a list of star ratings.
fn star_ratings(input: &[u8]) -> IResult<&[u8], Vec<(FlagSet<Mods>, f64)>> {
    length_count(
        le_u32,
        map(int_double_pair, |(i, d)| {
            (FlagSet::<Mods>::new_truncated(i), d)
        }),
    )(input)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ranked_status_decoding_works() {
        use RankedStatus::*;

        assert_eq!(ranked_status(&[0]), Ok((&[][..], Unknown)));
        assert_eq!(ranked_status(&[1]), Ok((&[][..], Unsubmitted)));
        assert_eq!(ranked_status(&[2]), Ok((&[][..], Pending)));
        assert_eq!(ranked_status(&[4]), Ok((&[][..], Ranked)));
        assert_eq!(ranked_status(&[5]), Ok((&[][..], Approved)));
        assert_eq!(ranked_status(&[6]), Ok((&[][..], Qualified)));
        assert_eq!(ranked_status(&[7]), Ok((&[][..], Loved)));

        assert_eq!(
            ranked_status(&[10]),
            Err(nom::Err::Error(nom::error::Error {
                input: &[10][..],
                code: nom::error::ErrorKind::Switch
            }))
        );
    }

    #[test]
    fn int_double_pair_decoding_works() {
        let int: u32 = 100;
        let double: f64 = 1234.56;
        let extra = [0x01, 0x02, 0x03];

        let mut pair = Vec::new();
        pair.push(0x08);
        pair.extend_from_slice(&int.to_le_bytes());
        pair.push(0x0d);
        pair.extend_from_slice(&double.to_le_bytes());
        pair.extend_from_slice(&extra);

        let mut missing_front_tag = Vec::new();
        missing_front_tag.extend_from_slice(&int.to_le_bytes());
        missing_front_tag.extend_from_slice(&double.to_le_bytes());

        let mut missing_middle_tag = Vec::new();
        missing_middle_tag.push(0x08);
        missing_middle_tag.extend_from_slice(&int.to_le_bytes());
        missing_middle_tag.extend_from_slice(&double.to_le_bytes());

        assert_eq!(int_double_pair(&pair), Ok((&extra[..], ((int, double)))));

        assert_eq!(
            int_double_pair(&missing_front_tag),
            Err(nom::Err::Error(nom::error::Error {
                input: &missing_front_tag[..],
                code: nom::error::ErrorKind::Tag
            }))
        );

        assert_eq!(
            int_double_pair(&missing_middle_tag),
            Err(nom::Err::Error(nom::error::Error {
                input: &double.to_le_bytes()[..],
                code: nom::error::ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn timing_point_decoding_works() {
        let bpm: f64 = 180.0;
        let song_offset: f64 = 250.0;
        let inherited = true;

        let mut input = Vec::new();
        input.extend_from_slice(&bpm.to_le_bytes());
        input.extend_from_slice(&song_offset.to_le_bytes());
        input.push(0x01);

        // Extra data
        input.extend_from_slice(&[0x05, 0x06]);

        assert_eq!(
            timing_point(&input),
            Ok((
                &[0x05, 0x06][..],
                TimingPoint {
                    bpm,
                    song_offset,
                    inherited
                }
            ))
        );
    }

    #[test]
    fn star_ratings_decoding_works() {
        let ratings: Vec<(FlagSet<Mods>, f64)> =
            vec![(Mods::None.into(), 1.2), (Mods::NoFail.into(), 2.3)];
        let length = ratings.len() as u32;

        let mut input = length.to_le_bytes().to_vec();

        for (mods, rating) in ratings.iter() {
            input.push(0x08);
            input.extend_from_slice(&mods.bits().to_le_bytes());
            input.push(0x0d);
            input.extend_from_slice(&rating.to_le_bytes());
        }

        assert_eq!(star_ratings(&input), Ok((&[][..], ratings)));
    }
}
