//! Teriminal module.

#![allow(async_fn_in_trait)]

use core::fmt::Debug;

use crate::{Command, Read, Write, csi, write};

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
    IO: Read + Write,
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

    /// Execute command.
    pub async fn execute(&mut self, action: impl Command) -> Result<(), Error> {
        action.write(&mut self.io).await.ok();
        Ok(())
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
    ClearAll,
    ClearLine,
    ClearCursorUp,
    ClearCursorDown,
    ScrollUpBy(u16),
    ScrollDownBy(u16),
}

impl Command for Action {
    async fn write<WriterTy: Write>(&self, writer: &mut WriterTy) -> Result<(), WriterTy::Error> {
        match self {
            Action::ClearAll => write!(writer, csi!("2J")).await,
            Action::ClearLine => write!(writer, csi!("2K")).await,
            Action::ClearCursorUp => write!(writer, csi!("1J")).await,
            Action::ClearCursorDown => write!(writer, csi!("0J")).await,
            Action::ScrollUpBy(count) => write!(writer, "\x1b[{}S", count).await,
            Action::ScrollDownBy(count) => write!(writer, csi!("{}T"), count).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write;

    use speculoos::prelude::*;

    use super::*;

    impl crate::Write for String {
        type Error = std::fmt::Error;

        async fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
            self.write_str(str::from_utf8(data).unwrap())?;
            Ok(data.len())
        }
    }

    #[tokio::test]
    async fn it_should_write_clear_all_action() {
        let action = Action::ClearAll;
        let mut buffer = String::default();

        let result = action.write(&mut buffer).await;
        assert_that!(result).is_ok();
        assert_that!(buffer.as_str()).is_equal_to(csi!("2J"));
    }

    #[tokio::test]
    async fn it_should_write_scroll_up_by_action() {
        let action = Action::ScrollUpBy(32);
        let mut buffer = String::default();

        let result = action.write(&mut buffer).await;
        assert_that!(result).is_ok();
        assert_that!(buffer.as_str()).is_equal_to(csi!("32S"));
    }
}
