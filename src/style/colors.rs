//! Colors.

use core::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    Reset,

    DarkGrey,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    Black,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    Grey,

    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },

    Ansi(u8),
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Reset => Err(fmt::Error),

            Color::DarkGrey => write!(f, "5;8"),
            Color::Red => write!(f, "5;9"),
            Color::Green => write!(f, "5;10"),
            Color::Yellow => write!(f, "5;11"),
            Color::Blue => write!(f, "5;12"),
            Color::Magenta => write!(f, "5;13"),
            Color::Cyan => write!(f, "5;14"),
            Color::White => write!(f, "5;15"),

            Color::Black => write!(f, "5;0"),
            Color::DarkRed => write!(f, "5;1"),
            Color::DarkGreen => write!(f, "5;2"),
            Color::DarkYellow => write!(f, "5;3"),
            Color::DarkBlue => write!(f, "5;4"),
            Color::DarkMagenta => write!(f, "5;5"),
            Color::DarkCyan => write!(f, "5;6"),
            Color::Grey => write!(f, "5;7"),

            Color::Rgb { r, g, b } => write!(f, "2;{r};{g};{b}"),
            Color::Ansi(value) => write!(f, "5;{value}"),
        }
    }
}

pub struct Background(pub Color);

impl fmt::Display for Background {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == Color::Reset {
            write!(f, "49")
        } else {
            write!(f, "48;{}", self.0)
        }
    }
}

pub struct Foreground(pub Color);

impl fmt::Display for Foreground {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == Color::Reset {
            write!(f, "39")
        } else {
            write!(f, "38;{}", self.0)
        }
    }
}

pub struct Underline(pub Color);

impl fmt::Display for Underline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == Color::Reset {
            write!(f, "59")
        } else {
            write!(f, "58;{}", self.0)
        }
    }
}
