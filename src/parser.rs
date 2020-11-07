use nom::IResult;
use nom::bytes::complete::{take, tag, take_while};
use nom::character::is_space;

pub fn length_value(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, _) = take_while(is_space)(input)?;
    let (input, title) = tag("TITLE:")(input)?;
    Ok((input, title))
}