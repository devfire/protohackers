use nom::number::complete::be_u16;
use nom::multi::length_data;
use nom::bytes::complete::tag;

impl Plate {
    fn take_str(s: &[u8]) -> IResult<&[u8], &[u8]> {
        length_data(be_u8)(s)
      }

    fn take_timestamp(input: &str) -> IResult<&str, u32> {
        be_u16(input)
    }

    pub fn parse(input: &str) -> IResult<&str, MessageType::Plate> {
        let (input, (plate, timestamp)) = (take_str, take_timestamp).parse(input)?;

        Ok((input, Plate{plate,timestamp}))
    }
}

