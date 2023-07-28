//! Models for the main `osu.db` database file, which contains information on installed beatmaps.

use nom::{
    bytes::complete::tag,
    combinator::map,
    number::complete::{le_f64, le_u32},
    sequence::{preceded, tuple},
    IResult,
};

use crate::common::boolean;

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

/// Parses a integer-double pair found in `osu.db`.
fn int_double_pair(input: &[u8]) -> IResult<&[u8], (u32, f64)> {
    let (rest, int) = preceded(tag(&[0x08]), le_u32)(input)?;
    let (rest, double) = preceded(tag(&[0x0d]), le_f64)(rest)?;

    Ok((rest, (int, double)))
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

#[cfg(test)]
pub mod tests {
    use super::*;

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
}
