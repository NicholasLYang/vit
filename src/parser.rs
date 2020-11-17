use nom::bytes::complete::{tag, take_while, take_while1, take_while_m_n};
use nom::character::complete::{multispace0, space0};
use nom::character::{is_alphabetic, is_alphanumeric, is_digit, is_space};
use nom::error::{Error, ErrorKind, ParseError};
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::Err;
use nom::IResult;

#[derive(Debug)]
pub struct EdlFile {
    title: String,
    clips: Vec<Clip>,
}

#[derive(Debug)]
pub enum EditChannels {
    Audio1,            // A
    Audio1Video,       // B
    Video,             // V
    Audio2,            // A2
    Audio2Video,       // A2/V
    Audio1Audio2,      // AA
    Audio1Audio2Video, // AA/V
}

#[derive(Debug)]
pub enum EditType {
    Cut,
    Dissolve,
    Wipe([u8; 3]),
    KeyBackground,
    KeyForeground,
    KRemoveFromForeground,
}

#[derive(Debug)]
pub enum Reel {
    Index { index: u16, is_b_roll: bool },
    Black,
    Aux,
}

#[derive(Debug)]
pub struct Clip {
    edit_index: u16,
    reel: Reel,
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
        Err(Err::Error(Error::new(
            edit_index_str,
            ErrorKind::TakeWhile1,
        )))
    } else {
        let edit_index: u16 = ((edit_index_str[0] - ZERO_CHAR) * 100
            + (edit_index_str[1] - ZERO_CHAR) * 10
            + edit_index_str[2]
            - ZERO_CHAR) as u16;
        Ok(edit_index)
    }
}

pub fn parse_reel(reel_chars: &[u8]) -> Result<Reel, Err<Error<&[u8]>>> {
    if reel_chars.len() == 4
        && is_digit(reel_chars[0])
        && is_digit(reel_chars[1])
        && is_digit(reel_chars[2])
    {
        let tape_index: u16 =
            ((reel_chars[0] - ZERO_CHAR) * 100 + (reel_chars[1] - ZERO_CHAR) * 10 + reel_chars[2]
                - ZERO_CHAR) as u16;
        Ok(Reel::Index {
            index: tape_index,
            is_b_roll: reel_chars[3] == b'b',
        })
    } else if reel_chars[0] == b'B' && reel_chars[1] == b'L' {
        Ok(Reel::Black)
    } else if reel_chars[0] == b'A' && reel_chars[1] == b'X' {
        Ok(Reel::Black)
    } else {
        Err(Err::Error(Error::new(reel_chars, ErrorKind::TakeWhile1)))
    }
}

macro_rules! ws {
    ($arg:expr) => {
        delimited(space0, $arg, space0)
    };
}

pub fn parse_clip(input: &[u8]) -> IResult<&[u8], Clip> {
    let (input, edit_index) = ws!(take_while1(is_digit))(input)?;
    let (input, reel) = ws!(take_while1(is_alphanumeric))(input)?;
    let (input, edit_channels) = ws!(take_while1(is_alphabetic))(input)?;
    let (input, edit_type) = ws!(take_while1(is_alphabetic))(input)?;
    let (input, clip_start_time) = ws!(take_while1(is_alphabetic))(input)?;
    let (input, clip_end_time) = ws!(take_while1(is_alphabetic))(input)?;
    let (input, timeline_position) = ws!(take_while1(is_alphabetic))(input)?;
    let (input, timeline_position_end) = ws!(take_while1(is_alphabetic))(input)?;

    let edit_index = parse_edit_index(edit_index)?;
    Ok((
        input,
        Clip {
            edit_index,
            reel: parse_reel(reel)?,
        },
    ))
}

pub fn parse_edl_file(input: &[u8]) -> IResult<&[u8], EdlFile> {
    let (input, (title, _)) = pair(parse_title, tag("\n"))(input)?;
    let (input, clips) = many0(delimited(multispace0, parse_clip, multispace0))(input)?;
    Ok((input, EdlFile { title, clips }))
}
