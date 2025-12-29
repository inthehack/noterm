//! Parser.

use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::{anychar, char, digit1};
use nom::combinator::{map, map_res};
use nom::error::{Error, ErrorKind};
use nom::sequence::preceded;
use nom::{IResult, Parser};

use crate::events::{CursorEvent, Event, KeyCode, KeyEvent, KeyModifiers, ScreenEvent};

pub fn parse(input: &str) -> IResult<&str, Event> {
    alt((
        char('\r').map(|_| Event::Key(KeyCode::Enter.into())),
        char('\n').map(|_| Event::Key(KeyCode::Enter.into())),
        char('\t').map(|_| Event::Key(KeyCode::Tab.into())),
        char('\x7f').map(|_| Event::Key(KeyCode::Backspace.into())),
        parse_ss3_escape_code,
        parse_csi_escape_code,
        parse_ascii_code,
    ))
    .parse(input)
}

pub(crate) fn parse_ascii_code(input: &str) -> IResult<&str, Event> {
    map(anychar, |c| {
        if c.is_uppercase() {
            Event::Key(KeyEvent::from(KeyCode::Char(c)).with_modifiers(KeyModifiers::SHIFT))
        } else {
            Event::Key(KeyCode::Char(c).into())
        }
    })
    .parse(input)
}

pub(crate) fn parse_ss3_escape_code(input: &str) -> IResult<&str, Event> {
    preceded(
        tag("\x1bO"),
        alt((
            map(char('A'), |_| Event::Key(KeyCode::Up.into())),
            map(char('B'), |_| Event::Key(KeyCode::Down.into())),
            map(char('C'), |_| Event::Key(KeyCode::Right.into())),
            map(char('D'), |_| Event::Key(KeyCode::Left.into())),
            map(char('H'), |_| Event::Key(KeyCode::Home.into())),
            map(char('F'), |_| Event::Key(KeyCode::End.into())),
            map(char('P'), |_| Event::Key(KeyCode::Fn(1).into())),
            map(char('Q'), |_| Event::Key(KeyCode::Fn(2).into())),
            map(char('R'), |_| Event::Key(KeyCode::Fn(3).into())),
            map(char('S'), |_| Event::Key(KeyCode::Fn(4).into())),
        )),
    )
    .parse(input)
}

pub(crate) fn parse_csi_escape_code(input: &str) -> IResult<&str, Event> {
    preceded(
        tag("\x1b["),
        alt((
            map(char('A'), |_| Event::Key(KeyCode::Up.into())),
            map(char('B'), |_| Event::Key(KeyCode::Down.into())),
            map(char('C'), |_| Event::Key(KeyCode::Right.into())),
            map(char('D'), |_| Event::Key(KeyCode::Left.into())),
            map(char('H'), |_| Event::Key(KeyCode::Home.into())),
            map(char('F'), |_| Event::Key(KeyCode::End.into())),
            map(char('Z'), |_| {
                Event::Key(KeyEvent::from(KeyCode::BackTab).with_modifiers(KeyModifiers::SHIFT))
            }),
            map(char('I'), |_| Event::Screen(ScreenEvent::FocusGained)),
            map(char('O'), |_| Event::Screen(ScreenEvent::FocusLost)),
            parse_csi_cursor_escape_code.map(Event::Cursor),
            parse_csi_function_key.map(Event::Key),
        )),
    )
    .parse(input)
}

pub(crate) fn parse_csi_function_key(input: &str) -> IResult<&str, KeyEvent> {
    map_res(
        (
            alt((char('1'), char('2'))),
            map_res(digit1, |s: &str| s.parse::<u8>()),
            char('~'),
        ),
        |(d, n, _)| match (d, n) {
            ('1', 1..=5) => Ok(KeyCode::Fn(n).into()),
            ('1', 7..=9) => Ok(KeyCode::Fn(n - 1).into()),
            ('2', 0) => Ok(KeyCode::Fn(9).into()),
            ('2', 1) => Ok(KeyCode::Fn(10).into()),
            ('2', 3) => Ok(KeyCode::Fn(11).into()),
            ('2', 4) => Ok(KeyCode::Fn(12).into()),
            _ => Err(Error::new(input, ErrorKind::Digit)),
        },
    )
    .parse(input)
}

// pub(crate) fn parse_csi_special_key(input: &str) -> IResult<&str, KeyEvent> {
//     (
//         map_res(digit1, |s: &str| s.parse::<u8>()),
//         char(';'),
//         parse_csi_key_modifiers_and_kind,
//     )
//         .map(|(code, _, (modifiers, kind))| {})
//         .parse(input)
// }

// pub(crate) fn parse_csi_key_modifiers_and_kind(
//     input: &str,
// ) -> IResult<&str, (KeyModifiers, KeyEventKind)> {
// }

pub(crate) fn parse_csi_cursor_escape_code(input: &str) -> IResult<&str, CursorEvent> {
    (
        map_res(digit1, |s: &str| s.parse::<u16>()),
        char(';'),
        map_res(digit1, |s: &str| s.parse::<u16>()),
        char('R'),
    )
        .map(|(line, _, column, _)| CursorEvent::Updated { line, column })
        .parse(input)
}
