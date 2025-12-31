//! Style.

use core::fmt;

pub use attributes::{Attribute, AttributeSet};
pub use colors::Color;

use crate::{Command, csi};

pub mod attributes;
pub mod colors;

pub struct SetBackgroundColor(pub Color);

impl Command for SetBackgroundColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), colors::Background(self.0))
    }
}

pub struct SetForegroundColor(pub Color);

impl Command for SetForegroundColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), colors::Foreground(self.0))
    }
}

pub struct SetUnderlineColor(pub Color);

impl Command for SetUnderlineColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), colors::Underline(self.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Colors {
    bg: Option<Color>,
    fg: Option<Color>,
    ul: Option<Color>,
}

impl Colors {
    pub fn is_empty(&self) -> bool {
        self.bg.is_none() && self.fg.is_none() && self.ul.is_none()
    }
}

pub struct SetColors(pub Colors);

impl Command for SetColors {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        match (self.0.fg, self.0.bg) {
            (Some(fg), Some(bg)) => write!(writer, csi!("{};{}m"), fg, bg)?,
            (Some(fg), None) => write!(writer, csi!("{}m"), fg)?,
            (None, Some(bg)) => write!(writer, csi!("{}m"), bg)?,
            (None, None) => {}
        }

        if let Some(ul) = self.0.ul {
            write!(writer, csi!("{}m"), ul)?;
        }

        Ok(())
    }
}

pub struct ResetColor;

impl Command for ResetColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("0m"))
    }
}

pub struct SetAttribute(pub Attribute);

impl Command for SetAttribute {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), self.0)
    }
}

pub struct SetAttributes(pub AttributeSet);

impl Command for SetAttributes {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        for attr in self.0.iter() {
            SetAttribute(attr).write(writer)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Style {
    colors: Colors,
    attributes: AttributeSet,
}

impl Style {
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty() && self.attributes.is_empty()
    }

    pub fn colors(&self) -> Colors {
        self.colors
    }

    pub fn attributes(&self) -> AttributeSet {
        self.attributes
    }
}

pub struct SetStyle(pub Style);

impl Command for SetStyle {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        SetColors(self.0.colors).write(writer)?;
        SetAttributes(self.0.attributes).write(writer)?;
        Ok(())
    }
}

pub struct Print<ContentTy: fmt::Display>(pub ContentTy);

impl<ContentTy: fmt::Display> Command for Print<ContentTy> {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, "{}", self.0)
    }
}

pub struct StyledContent<ContentTy> {
    style: Style,
    content: ContentTy,
}

impl<ContentTy> StyledContent<ContentTy> {
    pub fn new(content: ContentTy) -> Self {
        StyledContent {
            style: Default::default(),
            content,
        }
    }
}

impl<ContentTy: fmt::Display> fmt::Display for StyledContent<ContentTy> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        SetStyle(self.style).write(f)?;
        Print(&self.content).write(f)?;

        if !self.style.attributes.is_empty() {
            ResetColor.write(f)?;
        } else {
            if self.style.colors.bg.is_some() {
                SetBackgroundColor(Color::Reset).write(f)?;
            }

            if self.style.colors.fg.is_some() || self.style.colors.ul.is_some() {
                SetForegroundColor(Color::Reset).write(f)?;
            }
        }

        Ok(())
    }
}

pub trait AsStyle {
    fn style(&self) -> &Style;
}

pub trait AsStyleMut {
    fn style_mut(&mut self) -> &mut Style;
}

impl<ContentTy> AsStyle for StyledContent<ContentTy> {
    fn style(&self) -> &Style {
        &self.style
    }
}

impl<ContentTy> AsStyleMut for StyledContent<ContentTy> {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

pub trait Stylized: Sized {
    type Styled: AsStyle + AsStyleMut;

    fn stylize(self) -> Self::Styled;

    fn with(self, foreground: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().colors.fg = Some(foreground);
        styled
    }

    fn on(self, background: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().colors.bg = Some(background);
        styled
    }

    fn underline(self, underline: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().colors.ul = Some(underline);
        styled
    }

    fn attribute(self, attribute: Attribute) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().attributes.insert(attribute);
        styled
    }

    fn attributes(self, attributes: AttributeSet) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().attributes.extend(attributes);
        styled
    }
}

impl Stylized for &'static str {
    type Styled = StyledContent<&'static str>;

    fn stylize(self) -> Self::Styled {
        StyledContent::new(self)
    }
}

impl<const SIZE: usize> Stylized for heapless::String<SIZE> {
    type Styled = StyledContent<heapless::String<SIZE>>;

    fn stylize(self) -> StyledContent<Self> {
        StyledContent::new(self)
    }
}

impl<ContentTy: fmt::Display> Stylized for StyledContent<ContentTy> {
    type Styled = Self;

    fn stylize(self) -> Self::Styled {
        self
    }
}
