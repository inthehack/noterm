//! Cursor.

use core::fmt;

use crate::{Command, csi};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorStyle {
    Default,
    BlinkingBlock,
    SteadyBlock,
    BlinkingUnderscore,
    SteadyUnderscore,
    BlinkingBar,
    SteadyBar,
}

impl fmt::Display for CursorStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CursorStyle::Default => write!(f, csi!("q")),

            CursorStyle::BlinkingBlock => write!(f, csi!("1q")),
            CursorStyle::SteadyBlock => write!(f, csi!("2q")),

            CursorStyle::BlinkingUnderscore => write!(f, csi!("3q")),
            CursorStyle::SteadyUnderscore => write!(f, csi!("4q")),

            CursorStyle::BlinkingBar => write!(f, csi!("5q")),
            CursorStyle::SteadyBar => write!(f, csi!("6q")),
        }
    }
}

pub struct Home;

impl Command for Home {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("H"))
    }
}

pub struct MoveTo(pub u16, pub u16);

impl Command for MoveTo {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{};{}H"), self.1 + 1, self.0 + 1)
    }
}

pub struct MoveUp(pub u16);

impl Command for MoveUp {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}A"), self.0)?;
        }
        Ok(())
    }
}

pub struct MoveDown(pub u16);

impl Command for MoveDown {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}B"), self.0)?;
        }
        Ok(())
    }
}

pub struct MoveRight(pub u16);

impl Command for MoveRight {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}C"), self.0)?;
        }
        Ok(())
    }
}

pub struct MoveLeft(pub u16);

impl Command for MoveLeft {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}D"), self.0)?;
        }
        Ok(())
    }
}

pub struct MoveToColumn(pub u16);

impl Command for MoveToColumn {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}G"), self.0)
    }
}

pub struct MoveToRow(pub u16);

impl Command for MoveToRow {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}d"), self.0)
    }
}

pub struct MoveToPreviousLine(pub u16);

impl Command for MoveToPreviousLine {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}F"), self.0)?;
        }
        Ok(())
    }
}

pub struct MoveToNextLine(pub u16);

impl Command for MoveToNextLine {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if self.0 > 0 {
            write!(writer, csi!("{}E"), self.0)?;
        }
        Ok(())
    }
}

pub struct GetPosition;

impl Command for GetPosition {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("6n"))
    }
}

pub struct SavePosition;

impl Command for SavePosition {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("s"))
    }
}

pub struct RestorePosition;

impl Command for RestorePosition {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("u"))
    }
}

pub struct Hide;

impl Command for Hide {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("?25l"))
    }
}

pub struct Show;

impl Command for Show {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("?25h"))
    }
}

pub struct DisableBlinking;

impl Command for DisableBlinking {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("?12l"))
    }
}

pub struct EnableBlinking;

impl Command for EnableBlinking {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("?12h"))
    }
}

pub struct SetCursorStyle(pub CursorStyle);

impl Command for SetCursorStyle {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, "{}", self.0)
    }
}
