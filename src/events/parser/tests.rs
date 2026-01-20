//! Parser tests.

use rstest::rstest;
use speculoos::prelude::*;

use crate::events::{Event, KeyCode, KeyEvent, KeyModifiers, parse};

#[rstest]
// SS3 Arrow keys.
#[case::xterm_ss3_arrow_keys("\x1bOA", Event::Key(KeyCode::Up.into()))]
#[case::xterm_ss3_arrow_keys("\x1bOB", Event::Key(KeyCode::Down.into()))]
#[case::xterm_ss3_arrow_keys("\x1bOC", Event::Key(KeyCode::Right.into()))]
#[case::xterm_ss3_arrow_keys("\x1bOD", Event::Key(KeyCode::Left.into()))]
#[case::xterm_ss3_arrow_keys("\x1bOH", Event::Key(KeyCode::Home.into()))]
#[case::xterm_ss3_arrow_keys("\x1bOF", Event::Key(KeyCode::End.into()))]
// SS3 Function keys.
#[case::xterm_ss3_function_keys("\x1bOP", Event::Key(KeyCode::Fn(1).into()))]
#[case::xterm_ss3_function_keys("\x1bOQ", Event::Key(KeyCode::Fn(2).into()))]
#[case::xterm_ss3_function_keys("\x1bOR", Event::Key(KeyCode::Fn(3).into()))]
#[case::xterm_ss3_function_keys("\x1bOS", Event::Key(KeyCode::Fn(4).into()))]
// CSI Arrow keys.
#[case::xterm_csi_arrow_keys("\x1b[A", Event::Key(KeyCode::Up.into()))]
#[case::xterm_csi_arrow_keys("\x1b[B", Event::Key(KeyCode::Down.into()))]
#[case::xterm_csi_arrow_keys("\x1b[C", Event::Key(KeyCode::Right.into()))]
#[case::xterm_csi_arrow_keys("\x1b[D", Event::Key(KeyCode::Left.into()))]
#[case::xterm_csi_arrow_keys("\x1b[H", Event::Key(KeyCode::Home.into()))]
#[case::xterm_csi_arrow_keys("\x1b[F", Event::Key(KeyCode::End.into()))]
// CSI Function keys.
#[case::xterm_csi_function_keys("\x1b[11~", Event::Key(KeyCode::Fn(1).into()))]
#[case::xterm_csi_function_keys("\x1b[12~", Event::Key(KeyCode::Fn(2).into()))]
#[case::xterm_csi_function_keys("\x1b[13~", Event::Key(KeyCode::Fn(3).into()))]
#[case::xterm_csi_function_keys("\x1b[14~", Event::Key(KeyCode::Fn(4).into()))]
#[case::xterm_csi_function_keys("\x1b[15~", Event::Key(KeyCode::Fn(5).into()))]
#[case::xterm_csi_function_keys("\x1b[17~", Event::Key(KeyCode::Fn(6).into()))]
#[case::xterm_csi_function_keys("\x1b[18~", Event::Key(KeyCode::Fn(7).into()))]
#[case::xterm_csi_function_keys("\x1b[19~", Event::Key(KeyCode::Fn(8).into()))]
#[case::xterm_csi_function_keys("\x1b[20~", Event::Key(KeyCode::Fn(9).into()))]
#[case::xterm_csi_function_keys("\x1b[21~", Event::Key(KeyCode::Fn(10).into()))]
#[case::xterm_csi_function_keys("\x1b[23~", Event::Key(KeyCode::Fn(11).into()))]
#[case::xterm_csi_function_keys("\x1b[24~", Event::Key(KeyCode::Fn(12).into()))]
// CSI Function keys with modifiers.
#[case::xterm_csi_function_keys_with_modifiers(
    "\x1b[15;2~]",
    Event::Key(KeyEvent::from(KeyCode::Fn(5)).with_modifiers(KeyModifiers::SHIFT)),
)]
#[case::xterm_csi_function_keys_with_modifiers(
    "\x1b[15;3~]",
    Event::Key(KeyEvent::from(KeyCode::Fn(5)).with_modifiers(KeyModifiers::ALT)),
)]
#[case::xterm_csi_function_keys_with_modifiers(
    "\x1b[15;5~]",
    Event::Key(KeyEvent::from(KeyCode::Fn(5)).with_modifiers(KeyModifiers::CONTROL)),
)]
#[case::xterm_csi_function_keys_with_modifiers(
    "\x1b[15;9~]",
    Event::Key(KeyEvent::from(KeyCode::Fn(5)).with_modifiers(KeyModifiers::SUPER)),
)]
// Kitty keyboard protocol with no modifiers.
#[case::kitty_csi_unambiguous_key("\x1b[97u", Event::Key(KeyEvent::from(KeyCode::Char('a'))))]
// Kitty keyboard protocol with modifiers.
#[case::kitty_csi_unambiguous_key(
    "\x1b[97;1u",
    Event::Key(KeyEvent::from(KeyCode::Char('a')).with_modifiers(KeyModifiers::SHIFT)),
)]
#[case::kitty_csi_unambiguous_key(
    "\x1b[97;2u",
    Event::Key(KeyEvent::from(KeyCode::Char('a')).with_modifiers(KeyModifiers::ALT)),
)]
#[case::kitty_csi_unambiguous_key(
    "\x1b[97;4u",
    Event::Key(KeyEvent::from(KeyCode::Char('a')).with_modifiers(KeyModifiers::CONTROL)),
)]
#[case::kitty_csi_unambiguous_key(
    "\x1b[97;8u",
    Event::Key(KeyEvent::from(KeyCode::Char('a')).with_modifiers(KeyModifiers::SUPER)),
)]
#[case::kitty_csi_unambiguous_key(
    "\x1b[97;1;65u",
    Event::Key(KeyEvent::from(KeyCode::Char('a')).with_modifiers(KeyModifiers::SHIFT)),
)]
// Kitty keyboard protocol with included modifiers into the associated codepoint.
#[case::kitty_csi_unambiguous_key("\x1b[0;;229u", Event::Key(KeyEvent::from(KeyCode::Char('å'))))]
fn it_should_parse_single_event(#[case] input: &str, #[case] expected: Event) {
    assert_that!(parse(input))
        .is_ok()
        .map(|(_, second)| second)
        .is_equal_to(expected);
}

#[rstest]
#[case::digit("0", Event::Key(KeyEvent::from(KeyCode::Char('0'))))]
#[case::lower("a", Event::Key(KeyEvent::from(KeyCode::Char('a'))))]
#[case::upper(
    "A",
    Event::Key(KeyEvent::from(KeyCode::Char('a')).with_modifiers(KeyModifiers::SHIFT))
)]
fn it_should_parse_utf8_text(#[case] input: &str, #[case] expected: Event) {
    assert_that!(parse(input))
        .is_ok()
        .map(|(_, second)| second)
        .is_equal_to(expected);
}
