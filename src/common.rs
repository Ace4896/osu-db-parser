use nom::{
    bytes::complete::{take, take_while},
    combinator::{fail, map, map_res},
    number::complete::{le_u64, u8},
    IResult,
};
use time::{macros::datetime, Duration, OffsetDateTime};

pub type OsuStr<'a> = Option<&'a str>;

/// Parses a boolean value in osu!'s database file formats.
pub fn boolean(input: &[u8]) -> IResult<&[u8], bool> {
    map(u8, |byte| byte != 0)(input)
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
pub fn osu_string<'a>(input: &'a [u8]) -> IResult<&'a [u8], OsuStr<'a>> {
    let (i, head) = u8(input)?;

    match head {
        0x00 => Ok((i, None)),
        0x0b => {
            let (i, length) = uleb128(i)?;
            map_res(take(length), std::str::from_utf8)(i)
                .map(|(i, string)| (i, Some(string)))
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
        let test_string = "test";
        let mut test_string_bytes = vec![0x0b, 0x04];

        for byte in test_string.as_bytes() {
            test_string_bytes.push(*byte);
        }

        // Append a few other bytes that aren't part of the string
        test_string_bytes.push(0x01);
        test_string_bytes.push(0x02);
        test_string_bytes.push(0x03);

        assert_eq!(osu_string(&empty), Ok((&[][..], None)));
        assert_eq!(osu_string(&zero_length), Ok((&[][..], Some(""))));
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
