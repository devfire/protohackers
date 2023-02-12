use nom::{
    bytes::complete::{tag, take},
    combinator::{map, map_res},
    error::{Error, ErrorKind, ParseError},
    number::complete::{be_i32, be_u16, be_u32, be_u64},
    sequence::{pair, tuple},
    IResult,
};

#[derive(Debug)]
enum MessageType {
    TypeA {
        field1: i32,
        field2: u16,
    },
    TypeB {
        field1: String,
        field2: u64,
    },
    TypeC {
        field1: i32,
        field2: i32,
        field3: i32,
    },
    TypeD {
        field1: u32,
    },
}

fn parse_message_type_a(input: &[u8]) -> IResult<&[u8], MessageType> {
    map(
        pair(be_i32, be_u16),
        |(field1, field2)| MessageType::TypeA { field1, field2 },
    )(input)
}

fn parse_message_type_b(input: &[u8]) -> IResult<&[u8], MessageType> {
    map(
        pair(
            map_res(be_u32, |len| {
                // Ensure the length of the string is reasonable
                if len > 10000 {
                    Err(Error::new(input, ErrorKind::LengthValue))
                } else {
                    Ok(len)
                }
            }),
            map_res(take, |bytes: &[u8]| String::from_utf8(bytes.to_vec())),
        ),
        |(field1, field2)| MessageType::TypeB {
            field1: field2.unwrap_or_default(),
            field2: field1 as u64,
        },
    )(input)
}

fn parse_message_type_c(input: &[u8]) -> IResult<&[u8], MessageType> {
    map(
        tuple((be_i32, be_i32, be_i32)),
        |(field1, field2, field3)| MessageType::TypeC {
            field1,
            field2,
            field3,
        },
    )(input)
}

fn parse_message_type_d(input: &[u8]) -> IResult<&[u8], MessageType> {
    map(be_u32, |field1| MessageType::TypeD { field1 })(input)
}

fn parse_message(input: &[u8]) -> IResult<&[u8], MessageType> {
    // Parse the message type byte
    let (input, message_type_byte) = be_u32(input)?;
    match message_type_byte {
        1 => parse_message_type_a(input),
        2 => parse_message_type_b(input),
        3 => parse_message_type_c(input),
        4 => parse_message_type_d(input),
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Alt))),
    }
}
