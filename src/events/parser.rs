//! Parser.

// use core::pin::Pin;
// use core::task::{Context, Poll};

// use futures::{Stream, pin_mut};
// use heapless::Vec;
use nom::branch::alt;
use nom::bytes::complete::take_until;
use nom::bytes::streaming::tag;
use nom::character::streaming::{anychar, char, digit1};
use nom::combinator::{map, map_opt, map_res, opt};
use nom::error::{Error, ErrorKind};
use nom::sequence::{preceded, separated_pair, terminated};
use nom::{IResult, Parser as _};

use crate::events::{
    CursorEvent, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ScreenEvent,
};
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
        map(char('\r'), |_| Event::Key(KeyEvent::from(KeyCode::Enter))),
        map(char('\n'), |_| Event::Key(KeyEvent::from(KeyCode::Enter))),
        map(char('\t'), |_| Event::Key(KeyEvent::from(KeyCode::Tab))),
        map(char('\x7f'), |_| {
            Event::Key(KeyEvent::from(KeyCode::Backspace))
        }),
        parse_ctrl_modifier,
        map(parse_ascii, Event::Key),
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

pub(crate) fn parse_ascii(input: &str) -> IResult<&str, KeyEvent> {
    alt((map(anychar, |c| {
        if c.is_uppercase() {
            KeyEvent::from(KeyCode::Char((c as u8 - b'A' + b'a') as char))
                .with_modifiers(KeyModifiers::SHIFT)
        } else {
            KeyEvent::from(KeyCode::Char(c))
        }
    }),))
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
            map(parse_csi_function_key, Event::Key),
            map(parse_csi_special_key_code, Event::Key),
            map(parse_csi_u_encoded_escape_code, Event::Key),
            map(parse_csi_cursor_escape_code, Event::Cursor),
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

pub(crate) fn parse_csi_cursor_escape_code(input: &str) -> IResult<&str, CursorEvent> {
    terminated(
        separated_pair(
            map_res(digit1, |s: &str| s.parse::<u16>()),
            char(';'),
            map_res(digit1, |s: &str| s.parse::<u16>()),
        ),
        char('R'),
    )
    .map(|(row, column)| CursorEvent::Updated { row, column })
    .parse(input)
}

pub(crate) fn parse_csi_special_key_code(input: &str) -> IResult<&str, KeyEvent> {
    terminated(
        map(
            (
                map_opt(digit1, |s: &str| {
                    s.parse::<u8>()
                        .ok()
                        .and_then(parse_csi_special_key_code_value)
                        .map(KeyEvent::from)
                }),
                preceded(char(';'), parse_csi_u_modifiers),
            ),
            |(key_event, (key_modifiers, key_event_kind))| {
                key_event
                    .with_modifiers(key_modifiers)
                    .with_kind(key_event_kind)
            },
        ),
        char('~'),
    )
    .parse(input)
}

pub(crate) fn parse_csi_special_key_code_value(value: u8) -> Option<KeyCode> {
    match value {
        1 | 7 => Some(KeyCode::Home),
        2 => Some(KeyCode::Insert),
        3 => Some(KeyCode::Delete),
        4 | 8 => Some(KeyCode::End),
        5 => Some(KeyCode::PageUp),
        6 => Some(KeyCode::PageDown),
        n @ 11..=15 => Some(KeyCode::Fn(n - 10)),
        n @ 17..=21 => Some(KeyCode::Fn(n - 11)),
        n @ 23..=26 => Some(KeyCode::Fn(n - 12)),
        n @ 28..=29 => Some(KeyCode::Fn(n - 15)),
        n @ 31..=34 => Some(KeyCode::Fn(n - 17)),
        _ => None,
    }
}

pub(crate) fn parse_csi_u_encoded_escape_code(input: &str) -> IResult<&str, KeyEvent> {
    terminated(
        map(
            (
                parse_csi_u_codepoint,
                preceded(char(';'), parse_csi_u_modifiers),
                opt(preceded(char(';'), take_until("u"))),
            ),
            |(key_event, (key_modifiers, key_event_kind), _)| {
                key_event
                    .with_modifiers(key_modifiers)
                    .with_kind(key_event_kind)
            },
        ),
        char('u'),
    )
    .parse(input)
}

pub(crate) fn parse_csi_u_codepoint(input: &str) -> IResult<&str, KeyEvent> {
    map(
        (parse_ascii, opt(preceded(char(':'), digit1))),
        |(codepoint, _)| codepoint,
    )
    .parse(input)
}

pub(crate) fn parse_csi_u_modifiers(input: &str) -> IResult<&str, (KeyModifiers, KeyEventKind)> {
    let parsed: IResult<_, _> = separated_pair(
        map_res(digit1, |s: &str| {
            s.parse::<u8>().map(interpret_key_modifiers_from_mask)
        }),
        char(':'),
        map_res(digit1, |s: &str| {
            s.parse::<u8>().map(interpret_key_event_kind_from_value)
        }),
    )
    .parse(input);

    Ok(parsed.unwrap_or_else(|_| (input, (KeyModifiers::empty(), KeyEventKind::Pressed))))
}

pub(crate) fn interpret_key_modifiers_from_mask(mask: u8) -> KeyModifiers {
    let mut modifiers = KeyModifiers::empty();

    if mask & 1 != 0 {
        modifiers |= KeyModifiers::SHIFT;
    }

    if mask & 2 != 0 {
        modifiers |= KeyModifiers::ALT;
    }

    if mask & 4 != 0 {
        modifiers |= KeyModifiers::CONTROL;
    }

    if mask & 8 != 0 {
        modifiers |= KeyModifiers::SUPER;
    }

    if mask & 32 != 0 {
        modifiers |= KeyModifiers::META;
    }

    modifiers
}

pub(crate) fn interpret_key_event_kind_from_value(value: u8) -> KeyEventKind {
    match value {
        1 => KeyEventKind::Pressed,
        2 => KeyEventKind::Released,
        3 => KeyEventKind::Repeated,
        _ => KeyEventKind::Pressed,
    }
}
