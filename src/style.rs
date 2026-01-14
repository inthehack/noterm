//! Style.

use core::{fmt, marker::PhantomData};

use crate::{
    Command, csi,
    style::colors::{Background, Foreground, Underline},
};

pub mod attributes;
pub mod colors;

pub use attributes::{Attribute, AttributeSet};
pub use colors::Color;

pub struct SetBackgroundColor(pub Color);

impl Command for SetBackgroundColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), Background(self.0))
    }
}

pub struct SetForegroundColor(pub Color);

impl Command for SetForegroundColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), Foreground(self.0))
    }
}

pub struct SetUnderlineColor(pub Color);

impl Command for SetUnderlineColor {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        write!(writer, csi!("{}m"), Underline(self.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Colors {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub ul: Option<Color>,
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
            (Some(fg), Some(bg)) => write!(writer, csi!("{};{}m"), Foreground(fg), Background(bg))?,
            (Some(fg), None) => write!(writer, csi!("{}m"), Foreground(fg))?,
            (None, Some(bg)) => write!(writer, csi!("{}m"), Background(bg))?,
            (None, None) => {}
        }

        if let Some(ul) = self.0.ul {
            write!(writer, csi!("{}m"), Underline(ul))?;
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

pub struct StyledContent<'a, ContentTy> {
    content: ContentTy,
    style: Style,
    _marker: PhantomData<&'a ()>,
}

impl<'a, ContentTy: 'a> StyledContent<'a, ContentTy> {
    pub fn new(content: ContentTy) -> Self {
        StyledContent {
            content,
            style: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<ContentTy: fmt::Display> fmt::Display for StyledContent<'_, ContentTy> {
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

impl<ContentTy> AsStyle for StyledContent<'_, ContentTy> {
    fn style(&self) -> &Style {
        &self.style
    }
}

impl<ContentTy> AsStyleMut for StyledContent<'_, ContentTy> {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

macro_rules! stylized_attribute_impl {
    ($method:ident, $attribute:path) => {
        fn $method(self) -> Self::Styled {
            let mut styled = self.stylize();
            styled.style_mut().attributes.insert($attribute);
            styled
        }
    };
}

pub trait Stylized: Sized {
    type Styled: AsStyle + AsStyleMut;

    fn stylize(self) -> Self::Styled;

    fn fg(self, foreground: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().colors.fg = Some(foreground);
        styled
    }

    fn bg(self, background: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.style_mut().colors.bg = Some(background);
        styled
    }

    fn ul(self, underline: Color) -> Self::Styled {
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

    stylized_attribute_impl!(bold, Attribute::Bold);
    stylized_attribute_impl!(dim, Attribute::Dimmed);
    stylized_attribute_impl!(italic, Attribute::Italic);
    stylized_attribute_impl!(underline, Attribute::Underlined);
    stylized_attribute_impl!(strike, Attribute::Striked);
}

impl Stylized for char {
    type Styled = StyledContent<'static, char>;

    fn stylize(self) -> Self::Styled {
        StyledContent::new(self)
    }
}

impl<'a> Stylized for &'a str {
    type Styled = StyledContent<'a, &'a str>;

    fn stylize(self) -> Self::Styled {
        StyledContent::new(self)
    }
}

impl<const SIZE: usize> Stylized for heapless::String<SIZE> {
    type Styled = StyledContent<'static, heapless::String<SIZE>>;

    fn stylize(self) -> StyledContent<'static, Self> {
        StyledContent::new(self)
    }
}

impl<ContentTy: fmt::Display> Stylized for StyledContent<'_, ContentTy> {
    type Styled = Self;

    fn stylize(self) -> Self::Styled {
        self
    }
}
