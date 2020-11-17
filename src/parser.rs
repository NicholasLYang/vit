use nom::IResult;
use nom::bytes::complete::{tag, take_while, take_while1, take_while_m_n};
use nom::character::{is_space, is_alphabetic, is_digit};
use nom::error::{Error, ErrorKind, ParseError};
use nom::Err;
use nom::multi::many0;
use nom::sequence::{pair, delimited};
use nom::character::complete::multispace0;

#[derive(Debug)]
pub struct EdlFile {
    title: String,
    clips: Vec<Clip>
}

#[derive(Debug)]
pub enum EditChannels {
    Audio1, // A
    Audio1Video, // B
    Video, // V
    Audio2, // A2
    Audio2Video, // A2/V
    Audio1Audio2, // AA
    Audio1Audio2Video // AA/V
}

#[derive(Debug)]
pub enum EditType {
    Cut,
    Dissolve,
    Wipe([u8; 3]),
    KeyBackground,
    KeyForeground,
    KRemoveFromForeground
}

#[derive(Debug)]
pub struct Clip {
    edit_index: u16,
    tape: String,

}

static ZERO_CHAR: u8 = 48;

pub fn parse_title(input: &[u8]) -> IResult<&[u8], String> {
    let (input, _) = take_while(is_space)(input)?;
    let (input, _) = tag("TITLE:")(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, title) = take_while_m_n(0, 70, is_alphabetic)(input)?;
    take_while(is_space)(input)?;
    Ok((input, String::from_utf8_lossy(title).to_string()))
}

pub fn parse_edit_index(edit_index_str: &[u8]) -> Result<u16, Err<Error<&[u8]>>> {
    if edit_index_str.len() != 3 {
        Err(Err::Error(Error::new(edit_index_str, ErrorKind::TakeWhile1)))
    } else {
        let edit_index: u16 = ((edit_index_str[0] - ZERO_CHAR) * 100 +
            (edit_index_str[1] - ZERO_CHAR) * 10 +
            edit_index_str[2] - ZERO_CHAR) as u16;
        Ok(edit_index)
    }
}

pub fn parse_clip(input: &[u8]) -> IResult<&[u8], Clip> {
    let (input, _) = take_while(is_space)(input)?;
    let (input, edit_index) = take_while1(is_digit)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, tape) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, edit_channels) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, edit_type) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, clip_start_time) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, clip_end_time) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, timeline_position) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, timeline_position_end) = take_while1(is_alphabetic)(input)?;
    let (input, _) = take_while(is_space)(input)?;

    let edit_index = parse_edit_index(edit_index)?;
    Ok((input, Clip {
        edit_index,
        tape: String::from_utf8_lossy(tape).to_string(),
    }))
}

pub fn parse_edl_file(input: &[u8]) -> IResult<&[u8], EdlFile> {
    let (input, (title, _)) = pair(parse_title, tag("\n"))(input)?;
    let (input, clips) = many0(delimited(multispace0, parse_clip, multispace0))(input)?;
    Ok((input, EdlFile {
        title,
        clips
    }))
}