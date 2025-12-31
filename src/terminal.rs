//! Teriminal module.

#![allow(async_fn_in_trait)]
#![allow(dead_code)]

use core::fmt;

use crate::io;
use crate::{Command, csi};

/// Terminal config.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Size {
    pub rows: u16,
    pub columns: u16,
}

/// Terminal.
#[derive(Clone, Debug)]
pub struct Terminal<IO> {
    io: IO,
    size: Size,
    cursor: (u16, u16),
}

impl<IO> Terminal<IO>
where
    IO: io::Read + io::Write,
{
    /// Create a new terminal with the fiven I/O.
    pub fn new(io: IO) -> Self {
        Self::new_with_size(
            io,
            Size {
                rows: 25,
                columns: 80,
            },
        )
    }

    /// Create a new terminal with the given size.
    pub fn new_with_size(io: IO, size: Size) -> Self {
        Terminal {
            io,
            size,
            cursor: (0, 0),
        }
    }
}

/// Error.
#[derive(Clone, Debug, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Unknown.
    #[error("unkown error")]
    Unknown,
}

/// Action enums.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Action {
    ClearCursorDown,
    ClearCursorUp,
    ClearScreen,
    ClearHistory,
    ClearLineFromCursor,
    ClearLineToCursor,
    ClearLine,
    ScrollUp(u16),
    ScrollDown(u16),
    SetSize { width: u16, height: u16 },
    DisableLineWrapping,
    EnableLineWrapping,
}

impl Command for Action {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        match *self {
            Action::ClearCursorDown => writer.write_str(csi!("J")),
            Action::ClearCursorUp => writer.write_str(csi!("1J")),
            Action::ClearScreen => writer.write_str(csi!("2J")),
            Action::ClearHistory => writer.write_str(csi!("3J")),
            Action::ClearLineFromCursor => writer.write_str(csi!("K")),
            Action::ClearLineToCursor => writer.write_str(csi!("1K")),
            Action::ClearLine => writer.write_str(csi!("2K")),

            Action::ScrollUp(lines) if lines > 0 => write!(writer, csi!("{}S"), lines),
            Action::ScrollDown(lines) if lines > 0 => write!(writer, csi!("{}T"), lines),

            Action::SetSize { width, height } if width > 0 && height > 0 => {
                write!(writer, csi!(""))
            }

            Action::DisableLineWrapping => write!(writer, csi!("?7l")),
            Action::EnableLineWrapping => write!(writer, csi!("?7h")),

            Action::ScrollUp(_)
            | Action::ScrollDown(_)
            | Action::SetSize {
                width: _,
                height: _,
            } => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use speculoos::prelude::*;

    use crate::Executable;

    use super::*;

    impl crate::io::blocking::Write for String {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            self.push_str(str::from_utf8(data).unwrap());
            Ok(data.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn it_should_write_clear_all_action() {
        let action = Action::ClearScreen;
        let mut buffer = String::default();

        let result = buffer.execute(action);
        assert_that!(result).is_ok();
        assert_that!(buffer.as_str()).is_equal_to(csi!("2J"));
    }

    #[test]
    fn it_should_write_scroll_up_by_action() {
        let action = Action::ScrollUp(32);
        let mut buffer = String::default();

        let result = buffer.execute(action);
        assert_that!(result).is_ok();
        assert_that!(buffer.as_str()).is_equal_to(csi!("32S"));
    }
}
