//! Parser.

use core::num::ParseIntError;

use nom::branch::alt;
use nom::bytes::complete::take_until;
use nom::bytes::streaming::tag;
use nom::character::streaming::{anychar, char, digit1};
use nom::combinator::{map, map_opt, map_res, opt, success};
use nom::error::{Error, ErrorKind};
use nom::sequence::{preceded, separated_pair, terminated};
use nom::{IResult, Parser as _};

use crate::events::{
    CursorEvent, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ScreenEvent,
};

#[cfg(test)]
mod tests;

pub fn parse(input: &str) -> IResult<&str, Event> {
    #[cfg(feature = "defmt")]
    defmt::debug!("parse input: {}", input.as_bytes());

    alt((
        parse_xterm_ss3_escape_code,
        parse_xterm_csi_escape_code,
        map(parse_xterm_ctrl_escape_code, Event::Key),
        map(parse_xterm_csi_cursor_escape_code, Event::Cursor),
        map(parse_kitty_csi_escape_code, Event::Key),
        // Must be last before utf-8 catch-all, otherwise it catches part of valid patterns
        // without ALT modifiers.
        map(parse_xterm_alt_escape_code, Event::Key),
        // Must be the last as it acts as a catch-all for any non-control sequence.
        map(parse_utf8_char, Event::Key),
    ))
    .parse(input)
}

pub(crate) fn parse_xterm_alt_escape_code(input: &str) -> IResult<&str, KeyEvent> {
    map_res(preceded(char('\x1b'), parse), |event| {
        if let Event::Key(key_event) = event {
            Ok(key_event.with_modifiers(KeyModifiers::ALT).sanitize())
        } else {
            Err(Error::new(input, ErrorKind::Escaped))
        }
    })
    .parse(input)
}

pub(crate) fn parse_xterm_ctrl_escape_code(input: &str) -> IResult<&str, KeyEvent> {
    alt((
        map(char('\r'), |_| KeyEvent::from(KeyCode::Enter)),
        map(char('\n'), |_| KeyEvent::from(KeyCode::Enter)),
        map(char('\t'), |_| KeyEvent::from(KeyCode::Tab)),
        map(char('\x7f'), |_| KeyEvent::from(KeyCode::Backspace)),
        map_res(anychar, |c| {
            let keycode = match c {
                '\x01'..='\x1a' => KeyCode::Char((c as u8 - 0x1 + b'a') as char),
                '\x1c'..='\x1f' => KeyCode::Char((c as u8 - 0x1c + b'4') as char),
                '\0' => KeyCode::Char(' '),
                _ => return Err(Error::new(c, ErrorKind::Char)),
            };

            let key_event = if keycode == KeyCode::Char('h') {
                KeyEvent::from(KeyCode::Backspace)
            } else {
                KeyEvent::from(keycode).with_modifiers(KeyModifiers::CONTROL)
            };

            Ok(key_event.sanitize())
        }),
    ))
    .parse(input)
}

pub(crate) fn parse_utf8_char(input: &str) -> IResult<&str, KeyEvent> {
    map(anychar, |c| {
        if c.is_uppercase() {
            KeyEvent::from(KeyCode::Char((c as u8 - b'A' + b'a') as char))
                .with_modifiers(KeyModifiers::SHIFT)
                .sanitize()
        } else {
            KeyEvent::from(KeyCode::Char(c))
        }
    })
    .parse(input)
}

pub(crate) fn parse_utf8_codepoint(input: &str) -> IResult<&str, KeyCode> {
    map_opt(map_res(digit1, str::parse::<u32>), char::from_u32)
        .map(KeyCode::Char)
        .parse(input)
}

pub(crate) fn parse_xterm_ss3_escape_code(input: &str) -> IResult<&str, Event> {
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

pub(crate) fn parse_xterm_csi_escape_code(input: &str) -> IResult<&str, Event> {
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
                Event::Key(
                    KeyEvent::from(KeyCode::BackTab)
                        .with_modifiers(KeyModifiers::SHIFT)
                        .sanitize(),
                )
            }),
            map(char('I'), |_| Event::Screen(ScreenEvent::FocusGained)),
            map(char('O'), |_| Event::Screen(ScreenEvent::FocusLost)),
            map(
                preceded(char(';'), parse_csi_modifier_encoded_escape_code),
                Event::Key,
            ),
            map(parse_xterm_csi_function_key, Event::Key),
            map(parse_xterm_csi_cursor_escape_code, Event::Cursor),
            map(parse_xterm_vt220_csi_escape_code, Event::Key),
            // map(parse_csi_modifier_encoded_escape_code, Event::Key),
        )),
    )
    .parse(input)
}

pub(crate) fn parse_xterm_csi_function_key(input: &str) -> IResult<&str, KeyEvent> {
    map_res(
        terminated(
            (
                alt((char('1'), char('2'))),
                map_res(digit1, str::parse::<u8>),
                opt(preceded(char(';'), parse_xterm_csi_key_modifiers)),
            ),
            char('~'),
        ),
        |(d, n, key_modifiers_maybe)| {
            let key_event = match (d, n) {
                ('1', 1..=5) => KeyEvent::from(KeyCode::Fn(n)),
                ('1', 7..=9) => KeyEvent::from(KeyCode::Fn(n - 1)),
                ('2', 0) => KeyEvent::from(KeyCode::Fn(9)),
                ('2', 1) => KeyEvent::from(KeyCode::Fn(10)),
                ('2', 3) => KeyEvent::from(KeyCode::Fn(11)),
                ('2', 4) => KeyEvent::from(KeyCode::Fn(12)),
                _ => return Err(Error::new(input, ErrorKind::Digit)),
            };

            Ok(key_event.with_modifiers_maybe(key_modifiers_maybe))
        },
    )
    .parse(input)
}

pub(crate) fn parse_xterm_csi_cursor_escape_code(input: &str) -> IResult<&str, CursorEvent> {
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

pub(crate) fn parse_xterm_vt220_csi_escape_code(input: &str) -> IResult<&str, KeyEvent> {
    terminated(
        map(
            (
                map_opt(
                    map_res(digit1, |s: &str| s.parse::<u8>()),
                    interpret_xterm_vt220_csi_code_value,
                ),
                opt(preceded(char(';'), parse_xterm_csi_key_modifiers)),
            ),
            KeyEvent::from,
        ),
        char('~'),
    )
    .parse(input)
}

pub(crate) fn interpret_xterm_vt220_csi_code_value(value: u8) -> Option<KeyCode> {
    let keycode = match value {
        1 | 7 => KeyCode::Home,
        2 => KeyCode::Insert,
        3 => KeyCode::Delete,
        4 | 8 => KeyCode::End,
        5 => KeyCode::PageUp,
        6 => KeyCode::PageDown,
        n @ 11..=15 => KeyCode::Fn(n - 10),
        n @ 17..=21 => KeyCode::Fn(n - 11),
        n @ 23..=26 => KeyCode::Fn(n - 12),
        n @ 28..=29 => KeyCode::Fn(n - 15),
        n @ 31..=34 => KeyCode::Fn(n - 17),
        _ => return None,
    };

    Some(keycode)
}

pub(crate) fn parse_kitty_csi_escape_code(input: &str) -> IResult<&str, KeyEvent> {
    alt((
        preceded(
            tag("\x1b["),
            terminated(
                alt((
                    map(
                        preceded((char('0'), char(';'), char(';')), parse_utf8_codepoint),
                        KeyEvent::from,
                    ),
                    map(
                        (
                            parse_kitty_csi_codepoint,
                            opt(preceded(
                                char(';'),
                                alt((
                                    parse_kitty_csi_key_modifiers_and_kind,
                                    success((KeyModifiers::empty(), KeyEventKind::Pressed)),
                                )),
                            )),
                            opt(preceded(char(';'), digit1)),
                        ),
                        |(key_code, key_modifiers_and_kind, _)| {
                            KeyEvent::from((key_code, key_modifiers_and_kind))
                        },
                    ),
                )),
                char('u'),
            ),
        ),
        map(char('\x0d'), |_| KeyEvent::from(KeyCode::Enter)),
        map(alt((char('\x7f'), char('\x08'))), |_| {
            KeyEvent::from(KeyCode::Backspace)
        }),
        map(char('\x09'), |_| KeyEvent::from(KeyCode::Tab)),
    ))
    .parse(input)
}

pub(crate) fn parse_kitty_csi_codepoint(input: &str) -> IResult<&str, KeyCode> {
    map(
        (parse_utf8_codepoint, opt(preceded(char(':'), digit1))),
        |(key_code, _)| key_code,
    )
    .parse(input)
}

pub(crate) fn parse_xterm_csi_key_modifiers(input: &str) -> IResult<&str, KeyModifiers> {
    map_res(digit1, |s: &str| {
        s.parse::<u8>().map(interpret_xterm_key_modifiers_from_mask)
    })
    .parse(input)
}

pub(crate) fn interpret_xterm_key_modifiers_from_mask(mask: u8) -> KeyModifiers {
    interpret_kitty_key_modifiers_from_mask(mask.saturating_sub(1))
}

pub(crate) fn parse_kitty_csi_key_modifiers_and_kind(
    input: &str,
) -> IResult<&str, (KeyModifiers, KeyEventKind)> {
    (
        map_res(digit1, |s: &str| {
            s.parse::<u8>().map(interpret_kitty_key_modifiers_from_mask)
        }),
        alt((
            preceded(
                char(':'),
                map_res(digit1, |s: &str| {
                    s.parse::<u8>()
                        .map(interpret_kitty_key_event_kind_from_value)
                }),
            ),
            success(KeyEventKind::Pressed),
        )),
    )
        .parse(input)
}

pub(crate) fn interpret_kitty_key_modifiers_from_mask(mask: u8) -> KeyModifiers {
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

pub(crate) fn interpret_kitty_key_event_kind_from_value(value: u8) -> KeyEventKind {
    match value {
        1 => KeyEventKind::Pressed,
        2 => KeyEventKind::Released,
        3 => KeyEventKind::Repeated,
        _ => KeyEventKind::Pressed,
    }
}

pub(crate) fn parse_csi_modifier_encoded_escape_code(input: &str) -> IResult<&str, KeyEvent> {
    (
        preceded(
            take_until(";"),
            preceded(
                char(';'),
                alt((
                    parse_kitty_csi_key_modifiers_and_kind,
                    map_res(digit1, |s: &str| {
                        Ok::<_, ParseIntError>((
                            s.parse::<u8>()
                                .map(interpret_xterm_key_modifiers_from_mask)?,
                            KeyEventKind::Pressed,
                        ))
                    }),
                    success((KeyModifiers::empty(), KeyEventKind::Pressed)),
                )),
            ),
        ),
        map(
            map_opt(anychar, interpret_modifier_key_code_value),
            KeyEvent::from,
        ),
    )
        .map(|((key_modifiers, key_event_kind), key_event)| {
            key_event
                .with_modifiers(key_modifiers)
                .with_kind(key_event_kind)
                .sanitize()
        })
        .parse(input)
}

pub(crate) fn interpret_modifier_key_code_value(value: char) -> Option<KeyCode> {
    let keycode = match value {
        'A' => KeyCode::Up,
        'B' => KeyCode::Down,
        'C' => KeyCode::Right,
        'D' => KeyCode::Left,
        'F' => KeyCode::End,
        'H' => KeyCode::Home,
        'P' => KeyCode::Fn(1),
        'Q' => KeyCode::Fn(2),
        'R' => KeyCode::Fn(3),
        'S' => KeyCode::Fn(4),
        _ => return None,
    };

    Some(keycode)
}
