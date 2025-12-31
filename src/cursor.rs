//! Cursor.

use core::fmt;

use crate::{Command, csi};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Home,
    MoveTo { row: u16, column: u16 },
    MoveUp(u16),
    MoveDown(u16),
    MoveRight(u16),
    MoveLeft(u16),
    MoveToColumn(u16),
    MoveToRow(u16),
    MoveToPreviousLine(u16),
    MoveToNextLine(u16),
    GetPosition,
    SavePosition,
    RestorePosition,
    Hide,
    Show,
    DisableBlinking,
    EnableBlinking,
    SetCursorStyle(CursorStyle),
}

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

impl Command for Action {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        match *self {
            Action::Home => writer.write_str(csi!("H")),

            Action::MoveTo { row, column } => write!(writer, csi!("{};{}H"), row + 1, column + 1),
            Action::MoveUp(lines) if lines > 0 => write!(writer, csi!("{}A"), lines),
            Action::MoveDown(lines) if lines > 0 => write!(writer, csi!("{}B"), lines),
            Action::MoveRight(columns) if columns > 0 => write!(writer, csi!("{}C"), columns),
            Action::MoveLeft(columns) if columns > 0 => write!(writer, csi!("{}D"), columns),
            Action::MoveToColumn(column) => write!(writer, csi!("{}G"), column + 1),
            Action::MoveToRow(row) => write!(writer, csi!("{}d"), row + 1),
            Action::MoveToPreviousLine(lines) if lines > 0 => write!(writer, csi!("{}F"), lines),
            Action::MoveToNextLine(lines) if lines > 0 => write!(writer, csi!("{}E"), lines),

            Action::GetPosition => write!(writer, csi!("6n")),

            Action::SavePosition => write!(writer, csi!("s")),
            Action::RestorePosition => write!(writer, csi!("u")),

            Action::Hide => write!(writer, csi!("?25l")),
            Action::Show => write!(writer, csi!("?25h")),

            Action::DisableBlinking => write!(writer, csi!("?12l")),
            Action::EnableBlinking => write!(writer, csi!("?12h")),

            Action::SetCursorStyle(style) => write!(writer, "{}", style),

            Action::MoveUp(_)
            | Action::MoveDown(_)
            | Action::MoveRight(_)
            | Action::MoveLeft(_)
            | Action::MoveToPreviousLine(_)
            | Action::MoveToNextLine(_) => Ok(()),
        }
    }
}
