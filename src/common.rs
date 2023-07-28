use nom::{bytes::complete::take_while, number::complete::u8, IResult};

/// Decodes a ULEB128 value into an unsigned 64-bit integer.
pub fn uleb128(input: &[u8]) -> IResult<&[u8], u64> {
    let (rest, uleb_start) = take_while(|byte| byte & 0x80 > 1)(input)?;
    let (rest, uleb_final) = u8(rest)?;

    let mut result = 0;
    let mut shift = 0;

    for byte in uleb_start {
        result |= ((*byte & 0x7F) as u64) << shift;
        shift += 7;
    }

    result |= ((uleb_final & 0x7F) as u64) << shift;
    Ok((rest, result))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
