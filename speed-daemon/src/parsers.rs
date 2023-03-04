use nom::{
    branch::alt,
    bytes::streaming::{tag, take},
    error::{Error, ErrorKind},
    multi::{length_count, length_data},
    number::streaming::{be_u16, be_u32, be_u8, u8},
    sequence::delimited,
    Err, IResult,
};

use crate::message::MessageType;

fn parse_plate(input: &[u8]) -> nom::IResult<&[u8], MessageType> {
    // 0x20: Plate (Client->Server)
    let (input, _) = tag([0x20])(input)?;

    // Parse the length byte
    let (input, length) = be_u8(input)?;

    // Parse the string of `length` bytes
    let (input, string_bytes) = take(length)(input)?;

    // Convert the bytes to a String
    let string = String::from_utf8(string_bytes.to_vec()).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        ))
    })?;

    // timestamp: u32
    let (input, ts) = be_u32(input)?;

    // Return the plate and the timestamp
    Ok((input, MessageType::Plate { plate: string, timestamp: ts }))
}
