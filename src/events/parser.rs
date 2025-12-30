//! Parser.

// use core::pin::Pin;
// use core::task::{Context, Poll};

// use futures::{Stream, pin_mut};
// use heapless::Vec;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::character::streaming::{anychar, char, digit1};
use nom::combinator::{map, map_res, peek};
use nom::error::{Error, ErrorKind};
use nom::sequence::preceded;
use nom::{IResult, Parser as _};

use crate::events::{CursorEvent, Event, KeyCode, KeyEvent, KeyModifiers, ScreenEvent};
// use crate::io;

// pub struct Parser<ReaderTy> {
//     reader: ReaderTy,
//     buffer: Vec<u8, 32>,
//     cursor: usize,
// }

// impl<ReaderTy: io::Read> Parser<ReaderTy> {
//     pub fn new(reader: ReaderTy) -> Self {
//         Parser {
//             reader,
//             buffer: Default::default(),
//             cursor: 0,
//         }
//     }
// }

// impl<ReaderTy: io::Read + Unpin> Stream for Parser<ReaderTy>
// where
//     Self: Unpin,
// {
//     type Item = io::Result<Event>;

//     async fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let reader = Pin::new(&mut self.reader);
//         let fut = reader.read(&mut self.buffer[self.cursor..]);
//         pin_mut!(fut);

//         let amount = match fut.poll(cx) {
//             Poll::Pending => return Poll::Pending,
//             Poll::Ready(Ok(amount)) => amount,
//             Poll::Ready(Err(_)) => return Poll::Ready(Some(Err(io::Error::Unknown))),
//         };

//         self.cursor += amount;

//         let Ok(input) = str::from_utf8(&self.buffer[..self.cursor]) else {
//             self.cursor = 0;
//             return Poll::Pending;
//         };

//         match parse(input) {
//             Ok((rest, event)) => {
//                 if rest.is_empty() {
//                     self.cursor = 0;
//                 } else {
//                     self.cursor -= rest.len();
//                 }

//                 Poll::Ready(Some(Ok(event)))
//             }

//             Err(nom::Err::Incomplete(_)) => Poll::Pending,

//             Err(nom::Err::Error(_)) => {
//                 self.cursor = 0;
//                 Poll::Pending
//             }

//             Err(nom::Err::Failure(_)) => Poll::Ready(Some(Err(io::Error::Unknown))),
//         }
//     }
// }

pub fn parse(input: &str) -> IResult<&str, Event> {
    alt((
        parse_ss3_escape_code,
        parse_csi_escape_code,
        parse_alt_modifier,
        map(char('\r'), |_| Event::Key(KeyCode::Enter.into())),
        map(char('\n'), |_| Event::Key(KeyCode::Enter.into())),
        map(char('\t'), |_| Event::Key(KeyCode::Tab.into())),
        map(char('\x7f'), |_| Event::Key(KeyCode::Backspace.into())),
        parse_ctrl_modifier,
        parse_ascii_code,
    ))
    .parse(input)
}

pub(crate) fn parse_alt_modifier(input: &str) -> IResult<&str, Event> {
    map(preceded(char('\x1b'), parse), |event| {
        if let Event::Key(key_event) = event {
            Event::Key(key_event.with_modifiers(KeyModifiers::ALT))
        } else {
            event
        }
    })
    .parse(input)
}

pub(crate) fn parse_ctrl_modifier(input: &str) -> IResult<&str, Event> {
    map_res(anychar, |c| {
        let keycode = match c {
            '\x01'..='\x1a' => KeyCode::Char((c as u8 - 0x1 + b'a') as char),
            '\x1c'..='\x1f' => KeyCode::Char((c as u8 - 0x1c + b'4') as char),
            '\0' => KeyCode::Char(' '),
            _ => return Err(Error::new(c, ErrorKind::Char)),
        };

        Ok(Event::Key(
            KeyEvent::from(keycode).with_modifiers(KeyModifiers::CONTROL),
        ))
    })
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
            map(char('\x1b'), |_| Event::Key(KeyCode::Escape.into())),
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
