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
#[derive(Debug)]
pub struct Terminal<'a, WriterTy> {
    writer: &'a mut WriterTy,
    size: Size,
    cursor: (u16, u16),
}

impl<'a, WriterTy> Terminal<'a, WriterTy>
where
    WriterTy: io::blocking::Write,
{
    /// Create a new terminal with the fiven I/O.
    pub fn new(writer: &'a mut WriterTy) -> Self {
        Terminal {
            writer,
            size: Size {
                rows: 25,
                columns: 80,
            },
            cursor: (0, 0),
        }
    }

    /// Create a new terminal with the given size.
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Get inner writer.
    pub fn writer(&'a mut self) -> &'a mut WriterTy {
        self.writer
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClearType {
    All,
    History,
    CursorAndAbove,
    CursorAndBelow,
    Line,
    LineFromCursor,
    LineToCursor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clear(pub ClearType);

impl Command for Clear {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        match self.0 {
            ClearType::CursorAndBelow => write!(writer, csi!("J")),
            ClearType::CursorAndAbove => write!(writer, csi!("1J")),
            ClearType::All => write!(writer, csi!("2J")),
            ClearType::History => write!(writer, csi!("3J")),
            ClearType::LineFromCursor => write!(writer, csi!("K")),
            ClearType::LineToCursor => write!(writer, csi!("1K")),
            ClearType::Line => write!(writer, csi!("2K")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScrollUp(pub u16);

impl Command for ScrollUp {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}S"), self.0)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScrollDown(pub u16);

impl Command for ScrollDown {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}T"), self.0)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetSize(pub u16, pub u16);

impl Command for SetSize {
    fn write(&self, _writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 && self.1 > 0 {
            todo!()
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisableLineWrapping;

impl Command for DisableLineWrapping {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("?7l"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EnableLineWrapping;

impl Command for EnableLineWrapping {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("?7h"))
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
        let action = Clear(ClearType::All);
        let mut buffer = String::default();

        let result = buffer.execute(action);
        assert_that!(result).is_ok();
        assert_that!(buffer.as_str()).is_equal_to(csi!("2J"));
    }

    #[test]
    fn it_should_write_scroll_up_by_action() {
        let action = ScrollUp(32);
        let mut buffer = String::default();

        let result = buffer.execute(action);
        assert_that!(result).is_ok();
        assert_that!(buffer.as_str()).is_equal_to(csi!("32S"));
    }
}
