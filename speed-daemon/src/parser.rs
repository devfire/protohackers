use nom::{
    branch::alt,
    bytes::streaming::tag,
    error::{Error, ErrorKind},
    multi::{length_count, length_data},
    number::streaming::{be_u16, be_u32, u8},
    Err, IResult,
};

fn parse_plate (message_bytes: &[u8]) -> IResult<&[u8], MessageType::Plate> {
    
}