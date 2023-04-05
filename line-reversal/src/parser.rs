use nom::{
    branch::alt,
    bytes::streaming::{tag, take},
    multi::length_count,
    number::streaming::{be_u16, be_u32, be_u8, u8},
    IResult,
};

// use hex;
// use log::info;

use crate::message::MessageType;

fn parse_connect(input: &[u8]) -> nom::IResult<&[u8], MessageType> {
    // Connect: /connect/SESSION/
    // Grab the first /
    let (input, _) = tag("/")(input)?;

    let (input, _) = tag("connect")(input)?;

    let (input, _) = tag("/")(input)?;

    let (input, session) = be_u32(input)?;

    let (input, _) = tag("/")(input)?;

    // Return the plate and the timestamp
    Ok((input, MessageType::Connect { session }))
}

///
/// # Errors
///
/// This function will return an error if none of the parsers match.
pub fn parse_message(input: &[u8]) -> IResult<&[u8], MessageType> {
    // let hex_string = hex::encode(input);
    // info!("Parsing {}", hex_string);
    let (input, message) = alt((
        parse_connect,
    ))(input)?;
    // info!("Parser finished, inbound message: {:?}", message);
    Ok((input, message))
}
