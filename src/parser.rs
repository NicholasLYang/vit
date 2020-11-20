use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while, take_while1};
use nom::character::complete::{char, multispace0, space0};
use nom::character::{is_alphabetic, is_alphanumeric, is_digit, is_space};
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::Err;
use nom::IResult;

#[derive(Debug)]
pub struct EdlFile {
    title: String,
    lines: Vec<Line>,
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
    Wipe(u16),
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
    edit_channels: EditChannels,
    edit_type: EditType,
    transition_duration: Option<u16>,
    source_in_time: TimeCode,
    source_out_time: TimeCode,
    record_in_time: TimeCode,
    record_out_time: TimeCode,
}

#[derive(Debug)]
pub struct TimeCode {
    hours: u8,
    minutes: u8,
    seconds: u8,
    frames: u8,
}

#[derive(Debug)]
pub enum FrameCodeModeChange {
    DropFrame,
    NonDropFrame,
}

#[derive(Debug)]
pub enum Line {
    FCM(FrameCodeModeChange),
    Clip(Clip),
    Note(Vec<u8>),
}

static ZERO_CHAR: u8 = 48;

pub fn parse_title(input: &[u8]) -> IResult<&[u8], String> {
    let (input, _) = take_while(is_space)(input)?;
    let (input, _) = tag("TITLE:")(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, title) = take_until("\n")(input)?;
    Ok((input, String::from_utf8_lossy(title).trim().to_string()))
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
    if reel_chars.len() == 4 {
        let tape_index = parse_index(reel_chars)?;
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

pub fn parse_edit_channels(input: &[u8]) -> Result<EditChannels, Err<Error<&[u8]>>> {
    match input {
        b"A" => Ok(EditChannels::Audio1),
        b"B" => Ok(EditChannels::Audio1Video),
        b"V" => Ok(EditChannels::Video),
        b"A2" => Ok(EditChannels::Audio2),
        b"A2/V" => Ok(EditChannels::Audio2Video),
        b"AA" => Ok(EditChannels::Audio1Audio2),
        b"AA/V" => Ok(EditChannels::Audio1Audio2Video),
        _ => Err(Err::Error(Error::new(input, ErrorKind::TakeWhile1))),
    }
}

// Indices in CMX3600 are always 3 digit numbers
#[inline]
fn parse_index(input: &[u8]) -> Result<u16, Err<Error<&[u8]>>> {
    if input.len() == 3 && is_digit(input[0]) && is_digit(input[1]) && is_digit(input[2]) {
        Ok(
            ((input[0] - ZERO_CHAR) * 100 + (input[1] - ZERO_CHAR) * 10 + input[2] - ZERO_CHAR)
                as u16,
        )
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::TakeWhile1)))
    }
}

pub fn parse_edit_type(input: &[u8]) -> Result<EditType, Err<Error<&[u8]>>> {
    match input[0] {
        b'C' => Ok(EditType::Cut),
        b'D' => Ok(EditType::Dissolve),
        b'W' => Ok(EditType::Wipe(parse_index(&input[1..])?)),
        b'K' => {
            if input.len() == 1 {
                Ok(EditType::KeyForeground)
            } else if input[1] == b'B' {
                Ok(EditType::KeyBackground)
            } else if input[1] == b'O' {
                Ok(EditType::KRemoveFromForeground)
            } else {
                Err(Err::Error(Error::new(input, ErrorKind::TakeWhile1)))
            }
        }
        _ => Err(Err::Error(Error::new(input, ErrorKind::TakeWhile1))),
    }
}

#[inline]
fn parse_timecode_elem(input: &[u8], max: u8) -> Result<u8, Err<Error<&[u8]>>> {
    if input.len() == 2 {
        let elem = (input[0] - ZERO_CHAR) * 10 + input[1] - ZERO_CHAR;
        if elem > max {
            Err(Err::Error(Error::new(input, ErrorKind::TakeWhile1)))
        } else {
            Ok(elem)
        }
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::TakeWhile1)))
    }
}

fn parse_timecode(input: &[u8]) -> IResult<&[u8], TimeCode> {
    let (input, hours) = take_while1(is_alphanumeric)(input)?;
    let (input, _) = char(':')(input)?;
    let (input, minutes) = take_while1(is_alphanumeric)(input)?;
    let (input, _) = char(':')(input)?;
    let (input, seconds) = take_while1(is_alphanumeric)(input)?;
    let (input, _) = char(':')(input)?;
    let (input, frames) = take_while1(is_alphanumeric)(input)?;
    Ok((
        input,
        TimeCode {
            hours: parse_timecode_elem(hours, 23)?,
            minutes: parse_timecode_elem(minutes, 59)?,
            seconds: parse_timecode_elem(seconds, 59)?,
            frames: parse_timecode_elem(frames, 29)?,
        },
    ))
}

pub fn parse_clip(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, edit_index) = ws!(take_while1(is_digit))(input)?;
    let (input, reel) = ws!(take_while1(is_alphanumeric))(input)?;
    let (input, edit_channels) = ws!(take_while1(is_alphanumeric))(input)?;
    let (input, edit_type) = ws!(take_while1(is_alphabetic))(input)?;
    let edit_type = parse_edit_type(edit_type)?;
    let mut input = input;
    let transition_duration = match &edit_type {
        EditType::Cut => None,
        _ => {
            let res = ws!(take_while1(is_alphanumeric))(input)?;
            input = res.0;
            Some(parse_index(res.1)?)
        }
    };

    let (input, source_in_time) = ws!(parse_timecode)(input)?;
    let (input, source_out_time) = ws!(parse_timecode)(input)?;
    let (input, record_in_time) = ws!(parse_timecode)(input)?;
    let (input, record_out_time) = ws!(parse_timecode)(input)?;

    Ok((
        input,
        Line::Clip(Clip {
            edit_index: parse_edit_index(edit_index)?,
            reel: parse_reel(reel)?,
            edit_channels: parse_edit_channels(edit_channels)?,
            transition_duration,
            edit_type,
            source_in_time,
            source_out_time,
            record_in_time,
            record_out_time,
        }),
    ))
}

fn parse_fcm(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, _) = tag("FCM:")(input)?;
    let (input, mode) = ws!(alt((tag("DROP FRAME"), tag("NON-DROP FRAME"))))(input)?;
    Ok((
        input,
        Line::FCM(match mode {
            b"DROP FRAME" => FrameCodeModeChange::DropFrame,
            b"NON-DROP FRAME" => FrameCodeModeChange::NonDropFrame,
            _ => unreachable!(),
        }),
    ))
}

fn parse_notes(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, line) = take_until("\n")(input)?;
    Ok((input, Line::Note(line.into())))
}

pub fn parse_edl_file(input: &[u8]) -> IResult<&[u8], EdlFile> {
    let (input, title) = parse_title(input)?;
    let (input, _) = char('\n')(input)?;
    let (input, lines) = many0(delimited(
        multispace0,
        alt((parse_clip, parse_fcm, parse_notes)),
        multispace0,
    ))(input)?;
    Ok((input, EdlFile { title, lines }))
}
