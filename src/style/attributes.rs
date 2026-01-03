//! Attributes.

use core::fmt;
use core::ops::{BitAnd, BitOr, BitXor};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Attribute {
    #[default]
    Reset = 0,

    Bold = 1,
    Dimmed = 2,
    Italic = 3,
    Underlined = 4,
    Striked = 9,

    SlowBlink = 5,
    RapidBlink = 6,

    Reversed = 7,

    NotBoldOrDimmed = 22,
    NotItalic = 23,
    NotUnderlined = 24,
    NotStriked = 29,
    NotBlinking = 25,
    NotReversed = 27,
}

impl Attribute {
    pub fn bits(&self) -> u32 {
        if Attribute::Reset == *self {
            0
        } else {
            1u32 << self.index()
        }
    }

    pub(crate) fn index(&self) -> usize {
        ATTRIBUTE_LOOKUP_TABLE
            .iter()
            .position(|x| self == x)
            .expect("predifined attribute set")
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

static ATTRIBUTE_LOOKUP_TABLE: &[Attribute] = &[
    Attribute::Bold,
    Attribute::Dimmed,
    Attribute::Italic,
    Attribute::Underlined,
    Attribute::Striked,
    Attribute::SlowBlink,
    Attribute::RapidBlink,
    Attribute::Reversed,
    Attribute::NotBoldOrDimmed,
    Attribute::NotItalic,
    Attribute::NotUnderlined,
    Attribute::NotStriked,
    Attribute::NotReversed,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttributeSet {
    mask: u32,
}

impl AttributeSet {
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    pub fn reset(&mut self) -> &mut Self {
        self.mask = 0;
        self
    }

    pub fn insert(&mut self, attribute: Attribute) -> &mut Self {
        self.mask |= attribute.bits();
        self
    }

    pub fn remove(&mut self, attribute: Attribute) -> &mut Self {
        self.mask &= !attribute.bits();
        self
    }

    pub fn extend(&mut self, other: AttributeSet) -> &mut Self {
        self.mask |= other.mask;
        self
    }

    pub fn contains(&self, attribute: &Attribute) -> bool {
        (attribute.bits() & self.mask) != 0
    }

    pub fn intersection(&self, other: &AttributeSet) -> AttributeSet {
        AttributeSet {
            mask: self.mask & other.mask,
        }
    }

    pub fn union(&self, other: &AttributeSet) -> AttributeSet {
        AttributeSet {
            mask: self.mask | other.mask,
        }
    }

    pub fn difference(&self, other: &AttributeSet) -> AttributeSet {
        AttributeSet {
            mask: self.mask ^ other.mask,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Attribute> {
        ATTRIBUTE_LOOKUP_TABLE
            .iter()
            .copied()
            .filter(|attr| self.contains(attr))
    }
}

impl From<Attribute> for AttributeSet {
    fn from(value: Attribute) -> Self {
        *AttributeSet::default().insert(value)
    }
}

impl From<(Attribute, Attribute)> for AttributeSet {
    fn from(value: (Attribute, Attribute)) -> Self {
        *AttributeSet::from(value.0).insert(value.1)
    }
}

impl From<(Attribute, Attribute, Attribute)> for AttributeSet {
    fn from(value: (Attribute, Attribute, Attribute)) -> Self {
        *AttributeSet::from(value.0).insert(value.1).insert(value.2)
    }
}

impl From<(Attribute, Attribute, Attribute, Attribute)> for AttributeSet {
    fn from(value: (Attribute, Attribute, Attribute, Attribute)) -> Self {
        *AttributeSet::from(value.0)
            .insert(value.1)
            .insert(value.2)
            .insert(value.3)
    }
}

impl From<(Attribute, Attribute, Attribute, Attribute, Attribute)> for AttributeSet {
    fn from(value: (Attribute, Attribute, Attribute, Attribute, Attribute)) -> Self {
        *AttributeSet::from(value.0)
            .insert(value.1)
            .insert(value.2)
            .insert(value.3)
            .insert(value.4)
    }
}

impl
    From<(
        Attribute,
        Attribute,
        Attribute,
        Attribute,
        Attribute,
        Attribute,
    )> for AttributeSet
{
    fn from(
        value: (
            Attribute,
            Attribute,
            Attribute,
            Attribute,
            Attribute,
            Attribute,
        ),
    ) -> Self {
        *AttributeSet::from(value.0)
            .insert(value.1)
            .insert(value.2)
            .insert(value.3)
            .insert(value.4)
            .insert(value.5)
    }
}

impl BitAnd<Attribute> for Attribute {
    type Output = AttributeSet;

    fn bitand(self, rhs: Attribute) -> Self::Output {
        AttributeSet::from(self).intersection(&rhs.into())
    }
}

impl BitOr<Attribute> for Attribute {
    type Output = AttributeSet;

    fn bitor(self, rhs: Attribute) -> Self::Output {
        AttributeSet::from(self).union(&rhs.into())
    }
}

impl BitXor<Attribute> for Attribute {
    type Output = AttributeSet;

    fn bitxor(self, rhs: Attribute) -> Self::Output {
        AttributeSet::from(self).difference(&rhs.into())
    }
}

impl BitAnd<Attribute> for AttributeSet {
    type Output = AttributeSet;

    fn bitand(self, rhs: Attribute) -> Self::Output {
        self.intersection(&rhs.into())
    }
}

impl BitOr<Attribute> for AttributeSet {
    type Output = AttributeSet;

    fn bitor(self, rhs: Attribute) -> Self::Output {
        self.union(&rhs.into())
    }
}

impl BitXor<Attribute> for AttributeSet {
    type Output = AttributeSet;

    fn bitxor(self, rhs: Attribute) -> Self::Output {
        self.difference(&rhs.into())
    }
}

impl BitAnd<AttributeSet> for Attribute {
    type Output = AttributeSet;

    fn bitand(self, rhs: AttributeSet) -> Self::Output {
        AttributeSet::from(self).intersection(&rhs)
    }
}

impl BitOr<AttributeSet> for Attribute {
    type Output = AttributeSet;

    fn bitor(self, rhs: AttributeSet) -> Self::Output {
        AttributeSet::from(self).union(&rhs)
    }
}

impl BitXor<AttributeSet> for Attribute {
    type Output = AttributeSet;

    fn bitxor(self, rhs: AttributeSet) -> Self::Output {
        AttributeSet::from(self).difference(&rhs)
    }
}

impl BitAnd<AttributeSet> for AttributeSet {
    type Output = AttributeSet;

    fn bitand(self, rhs: AttributeSet) -> Self::Output {
        self.intersection(&rhs)
    }
}

impl BitOr<AttributeSet> for AttributeSet {
    type Output = AttributeSet;

    fn bitor(self, rhs: AttributeSet) -> Self::Output {
        self.union(&rhs)
    }
}

impl BitXor<AttributeSet> for AttributeSet {
    type Output = AttributeSet;

    fn bitxor(self, rhs: AttributeSet) -> Self::Output {
        self.difference(&rhs)
    }
}
