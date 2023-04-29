// use log::info;
use nom::{
    branch::alt,
    bytes::{
        complete::{escaped_transform, take_until},
        streaming::{is_not, tag},
    },
    character::complete::alpha1,
    combinator::{map_res, value},
    sequence::delimited,
    IResult,
};

// use hex;

use crate::{message::MessageType, types::SessionPosDataStruct};

fn parse_number_u32<'a>(
    first: &'a str,
    input_bytes: &'a [u8],
    third: &'a str,
) -> IResult<&'a [u8], u32> {
    // The `delimited` combinator runs a sequence
    // of three inner parsers and discards the
    // result of the first and third one if
    // they're successful.
    delimited(
        // `tag` is a parser that simply captures
        // the literal string it is passed.
        tag(first),
        // The `map_res` combinator applies a
        // closure to the output of a parser,
        // converting any errors returned from the
        // closure into nom errors.
        map_res(
            // In this case, our parser is simply
            // capturing anything that isn't the
            // ending delimiter, which should be
            // the number itself.
            is_not(third),
            // Since our bytes here aren't the raw number but rather a string of the
            // number (for instance, not the byte 0x04 itself but 0x34, which is the
            // ASCII for "4") we must parse it into one.
            |bytes| String::from_utf8_lossy(bytes).parse::<u32>(),
        ),
        tag("/"),
    )(input_bytes)
}

fn parse_connect(input: &[u8]) -> nom::IResult<&[u8], MessageType> {
    // info!("Parsing {:?}", input);
    // Connect: /connect/SESSION/
    // Grab the first /

    // NOTE: trailing / is not here, it is parsed immediately below
    let (input, _) = tag("/connect")(input)?;

    // this parses and extracts the u32 SESSION between a pair of forward slashes
    let (input, session) = parse_number_u32("/", input, "/")?;

    // Return the plate and the timestamp
    Ok((input, MessageType::Connect { session }))
}

fn parse_ack(input: &[u8]) -> nom::IResult<&[u8], MessageType> {
    // /ack/SESSION/LENGTH/
    // NOTE: trailing / is not here, it is parsed immediately below
    let (input, _) = tag("/ack")(input)?;

    // this parses and extracts the u32 SESSION between a pair of forward slashes
    let (input, session) = parse_number_u32("/", input, "/")?;

    // at this point, we have LENGTH/ left, so leading string is empty
    let (input, length) = parse_number_u32("", input, "/")?;

    Ok((input, MessageType::Ack { session, length }))
}

fn parse_data(input: &[u8]) -> nom::IResult<&[u8], MessageType> {
    // /data/SESSION/POS/DATA/
    // NOTE: trailing / is not here, it is parsed immediately below
    let (input, _) = tag("/data")(input)?;

    // this parses and extracts the u32 SESSION between a pair of forward slashes
    let (input, session) = parse_number_u32("/", input, "/")?;

    // at this point, we have POS/ left, so leading string is empty
    let (input, pos) = parse_number_u32("", input, "/")?;

    let (input, data) = take_until("/")(input)?;

    // this contains the original escaped string
    let data_string =
        String::from_utf8(data.to_vec()).expect("failed conversion from data to string");

    // // DATA is the string to be reversed.
    // let (input, data) =
    //     parse_data_string(std::str::from_utf8(input).expect("Unable to convert [u8] to str"))
    //         .expect("Unable to parse data string");

    let session_pos_data = SessionPosDataStruct::new(session, pos, data_string);

    // let input = input.as_bytes(); // back to byte array

    Ok((input, MessageType::Data { session_pos_data }))
}

///
/// # Errors
///
/// This function will return an error if none of the parsers match.
pub fn parse_message(input: &[u8]) -> IResult<&[u8], MessageType> {
    // let hex_string = hex::encode(input);
    // info!("Parsing {}", input);
    let (input, message) = alt((parse_connect, parse_ack, parse_data))(input)?;
    // info!("Parser finished, inbound message: {:?}", message);
    Ok((input, message))
}
