use flagset::{flags, FlagSet};
use nom::{
    bytes::complete::{take, take_while},
    combinator::{fail, map, map_res},
    number::complete::{le_u32, le_u64, u8},
    IResult,
};
use time::{macros::datetime, Duration, OffsetDateTime};

pub type OsuString = Option<String>;

/// Represents the different gameplay modes for a beatmap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayMode {
    Standard = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

flags! {
    /// Represents a combination of gameplay modifiers.
    pub enum Mods: u32 {
        None = 0,
        NoFail = 1 << 0,
        Easy = 1 << 1,
        TouchDevice = 1 << 2,
        Hidden = 1 << 3,
        HardRock = 1 << 4,
        SuddenDeath = 1 << 5,
        DoubleTime = 1 << 6,
        Relax = 1 << 7,
        HalfTime = 1 << 8,
        Nightcore = (1 << 6) | (1 << 9), // Always used with DT
        Flashlight = 1 << 10,
        Autoplay = 1 << 11,
        SpunOut = 1 << 12,
        Autopilot = 1 << 13, // a.k.a. Relax2
        Perfect = 1 << 14,
        Key4 = 1 << 15,
        Key5 = 1 << 16,
        Key6 = 1 << 17,
        Key7 = 1 << 18,
        Key8 = 1 << 19,
        KeyMod = (Mods::Key4 | Mods::Key5 | Mods::Key6 | Mods::Key7 | Mods::Key8).bits(),
        FadeIn = 1 << 20,
        Random = 1 << 21,
        Cinema = 1 << 22,
        TargetPractice = 1 << 23,
        Key9 = 1 << 24,
        Coop = 1 << 25,
        Key1 = 1 << 26,
        Key3 = 1 << 27,
        Key2 = 1 << 28,
        ScoreV2 = 1 << 29,
        Mirror = 1 << 30,
    }
}

/// Parses a boolean value in osu!'s database file formats.
pub fn boolean(input: &[u8]) -> IResult<&[u8], bool> {
    map(u8, |byte| byte != 0)(input)
}

/// Parses a gameplay mode value.
pub fn gameplay_mode(input: &[u8]) -> IResult<&[u8], GameplayMode> {
    use GameplayMode::*;

    let (i, status) = u8(input)?;
    let status = match status {
        0 => Standard,
        1 => Taiko,
        2 => Catch,
        3 => Mania,
        _ => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Switch,
            }))
        }
    };

    Ok((i, status))
}

/// Parses a set of gameplay modifiers.
pub fn modifiers(input: &[u8]) -> IResult<&[u8], FlagSet<Mods>> {
    map(le_u32, FlagSet::<Mods>::new_truncated)(input)
}

/// Decodes a ULEB128 value into an unsigned 64-bit integer.
pub fn uleb128(input: &[u8]) -> IResult<&[u8], u64> {
    let (i, uleb_start) = take_while(|byte| byte & 0x80 > 1)(input)?;
    let (i, uleb_final) = u8(i)?;

    let mut result = 0;
    let mut shift = 0;

    for byte in uleb_start {
        result |= ((*byte & 0x7F) as u64) << shift;
        shift += 7;
    }

    result |= ((uleb_final & 0x7F) as u64) << shift;
    Ok((i, result))
}

/// Decodes a string found in osu!'s database file formats.
///
/// - If the first byte is 0x00, then no string value is present.
/// - If the first byte is 0x0b, then this is followed by a ULEB128 value indicating the length, then the UTF-8 string bytes.
///
///  **NOTE**: There are two possible values for empty strings; these can be distinguished as follows:
///
/// - `0x00` => Empty string marker; output is `None`
/// - `0x0b, 0x00` => Zero length string; output is `Some("")`
pub fn osu_string(input: &[u8]) -> IResult<&[u8], OsuString> {
    let (i, head) = u8(input)?;

    match head {
        0x00 => Ok((i, None)),
        0x0b => {
            let (i, length) = uleb128(i)?;
            map(map_res(take(length), std::str::from_utf8), |s| {
                Some(s.to_string())
            })(i)
        }
        _ => fail(input),
    }
}

/// Parses a DateTime from .NET's [`DateTime.Ticks`](https://learn.microsoft.com/en-us/dotnet/api/system.datetime.ticks?view=netframework-4.7.2).
pub fn windows_datetime(input: &[u8]) -> IResult<&[u8], OffsetDateTime> {
    const WINDOWS_EPOCH: OffsetDateTime = datetime!(0001-01-01 0:00 UTC);

    // In .NET, there are 10,000 ticks per millisecond
    // So 10 ticks / microsecond, 0.01 ticks per nanosecond
    let (i, ticks) = le_u64(input)?;
    let result = WINDOWS_EPOCH
        + Duration::microseconds((ticks / 10) as i64)
        + Duration::nanoseconds(((ticks % 10) * 100) as i64);

    Ok((i, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_decoding_works() {
        // Any non-zero byte should result in true
        assert_eq!(boolean(&[0x00]), Ok(((&[][..]), false)));
        assert_eq!(boolean(&[0x01]), Ok(((&[][..]), true)));

        assert_eq!(
            boolean(&[0xFF, 0x01, 0x02]),
            Ok(((&[0x01, 0x02][..]), true))
        );
    }

    #[test]
    fn gameplay_mode_decoding_works() {
        use GameplayMode::*;

        assert_eq!(gameplay_mode(&[0]), Ok((&[][..], Standard)));
        assert_eq!(gameplay_mode(&[1]), Ok((&[][..], Taiko)));
        assert_eq!(gameplay_mode(&[2]), Ok((&[][..], Catch)));
        assert_eq!(gameplay_mode(&[3]), Ok((&[][..], Mania)));

        assert_eq!(
            gameplay_mode(&[10]),
            Err(nom::Err::Error(nom::error::Error {
                input: &[10][..],
                code: nom::error::ErrorKind::Switch
            }))
        );
    }

    #[test]
    fn uleb128_decoding_works() {
        // 0xE5, 0x8E, 0x26 ==> 624485
        // ULEB128 value by itself
        assert_eq!(uleb128(&[0xE5, 0x8E, 0x26]), Ok((&[][..], 624485)));

        // ULEB128 value followed by other bytes
        assert_eq!(
            uleb128(&[0xE5, 0x8E, 0x26, 0x80, 0x81, 0x82]),
            Ok((&[0x80, 0x81, 0x82][..], 624485))
        );

        // Empty value
        assert_eq!(
            uleb128(&[]),
            Err(nom::Err::Error(nom::error::Error {
                input: &[][..],
                code: nom::error::ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn osu_string_decoding_works() {
        let empty = vec![0x00];
        let zero_length = vec![0x0b, 0x00];
        let test_string = String::from("test");
        let mut test_string_bytes = vec![0x0b, 0x04];

        for byte in test_string.as_bytes() {
            test_string_bytes.push(*byte);
        }

        // Append a few other bytes that aren't part of the string
        test_string_bytes.push(0x01);
        test_string_bytes.push(0x02);
        test_string_bytes.push(0x03);

        assert_eq!(osu_string(&empty), Ok((&[][..], None)));
        assert_eq!(
            osu_string(&zero_length),
            Ok((&[][..], Some("".to_string())))
        );
        assert_eq!(
            osu_string(&test_string_bytes),
            Ok((&[0x01, 0x02, 0x03][..], Some(test_string)))
        );
    }

    #[test]
    fn windows_datetime_decoding_works() {
        // 07/28/2023 15:30:20 +00:00 ==> 638261550200000000 ticks
        let datetime = datetime!(2023-07-28 15:30:20 UTC);
        let ticks = 638261550200000000u64;

        // Should only parse the first 8 bytes
        let mut input = ticks.to_le_bytes().to_vec();
        input.push(0x01);
        input.push(0x02);
        input.push(0x03);

        assert_eq!(
            windows_datetime(&input),
            Ok((&[0x01, 0x02, 0x03][..], datetime))
        );
    }
}
