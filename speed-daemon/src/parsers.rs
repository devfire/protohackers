use nom::{
    branch::alt,
    bytes::streaming::{tag, take},
    error::{Error, ErrorKind},
    multi::{length_count, length_data, count},
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
    let plate = String::from_utf8(string_bytes.to_vec()).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        ))
    })?;

    // timestamp: u32
    let (input, timestamp) = be_u32(input)?;

    // Return the plate and the timestamp
    Ok((input, MessageType::Plate { plate, timestamp }))
}

pub fn parse_want_heartbeat (input: &[u8]) -> IResult<&[u8], MessageType> {
    //0x40: WantHeartbeat (Client->Server)
    let (input, _) = tag([0x40])(input)?;
    let (input, interval) = be_u32(input)?;
    Ok((input, MessageType::WantHeartbeat{interval}))
}

pub fn parse_i_am_camera (input: &[u8]) -> IResult<&[u8], MessageType> {
    //0x80: IAmCamera (Client->Server)
    let (input, _) = tag([0x80])(input)?;
    let (input, road) = be_u16(input)?;
    let (input, mile) = be_u16(input)?;
    let (input, limit) = be_u16(input)?;
    Ok((input, MessageType::IAmCamera { road, mile, limit}))
}

pub fn parse_i_am_dispatcher (input: &[u8]) -> IResult<&[u8], MessageType> {
    // 0x81: IAmDispatcher (Client->Server)
    let (input, _) = tag([0x81])(input)?;
    let (input, numroads)  = be_u8(input)?;
    let (input, roads) = length_count(u8, be_u16)(input)?;
    Ok((input, MessageType::IAmDispatcher { numroads, roads }))
}


pub fn parse_message(input: &[u8]) -> IResult<&[u8], MessageType> {
    let (input, message) = alt((
        parse_plate,
        parse_want_heartbeat,
        parse_i_am_camera,
        parse_i_am_dispatcher,
    ))(input)?;
    Ok((input, message))
}