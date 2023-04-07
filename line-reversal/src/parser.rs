use log::info;
use nom::{
    branch::alt,
    bytes::{streaming::{tag}, complete::is_not},
    multi::{many1, many0},
    number::streaming::{ be_u32},
    IResult, sequence::{terminated, delimited}, combinator::{recognize, map_res}, character::streaming::{one_of,char}
    
};

use hex;


use crate::message::MessageType;

fn parse_number_u32(input_bytes: &[u8])
    -> IResult<&[u8], u32> {
    // The `delimited` combinator runs a sequence
    // of three inner parsers and discards the
    // result of the first and third one if
    // they're successful.
    delimited(
        // `tag` is a parser that simply captures
        // the literal string it is passed.
        tag("/"),
        // The `map_res` combinator applies a
        // closure to the output of a parser,
        // converting any errors returned from the
        // closure into nom errors.
        map_res(
            // In this case, our parser is simply
            // capturing anything that isn't the
            // ending delimiter, which should be
            // the number itself.
            is_not("/"),
            // Since our bytes here aren't the raw
            // number but rather a string of the
            // number (for instance, not the byte
            // 0x04 itself but 0x34, which is the
            // ASCII for "4") we must parse it
            // into one.
            |bytes| String::from_utf8_lossy(bytes).parse::<u32>()
        ),
        tag("/")
    )(input_bytes)
}


  
fn parse_connect(input: &[u8]) -> nom::IResult<&[u8], MessageType> {
    // info!("Parsing {:?}", input);
    // Connect: /connect/SESSION/
    // Grab the first /
    
    // NOTE: trailing / is not here, it is parsed immediately below
    let (input, _) = tag("/connect")(input)?;


    // this parses and extracts the u32 SESSION between a pair of forward slashes
    let (input, session) = parse_number_u32(input)?;

    info!("parse_connect session: {session}");
    
    // Return the plate and the timestamp
    Ok((input, MessageType::Connect { session }))
}

///
/// # Errors
///
/// This function will return an error if none of the parsers match.
pub fn parse_message(input: &[u8]) -> IResult<&[u8], MessageType> {
    // let hex_string = hex::encode(input);
    // info!("Parsing {}", input);
    let (input, message) = alt((parse_connect,))(input)?;
    info!("Parser finished, inbound message: {:?}", message);
    Ok((input, message))
}
