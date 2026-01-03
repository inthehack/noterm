//! Events.

use bitflags::bitflags;

pub mod parser;
pub mod stream;

// pub use parser::Parser;
pub use parser::parse;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    Cursor(CursorEvent),
    Key(KeyEvent),
    Screen(ScreenEvent),
}

impl Event {
    pub fn is_screen_event(&self) -> bool {
        matches!(self, Event::Screen(_))
    }

    pub fn as_screen_event(&self) -> Option<&ScreenEvent> {
        if let Event::Screen(event) = &self {
            return Some(event);
        }

        None
    }

    pub fn is_key_event(&self) -> bool {
        matches!(self, Event::Key(_))
    }

    pub fn as_key_event(&self) -> Option<&KeyEvent> {
        if let Event::Key(event) = &self {
            return Some(event);
        }

        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CursorEvent {
    Updated { row: u16, column: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

impl KeyEvent {
    pub fn new(code: KeyCode) -> Self {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Pressed,
        }
    }

    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers |= modifiers;
        self
    }

    pub fn with_modifiers_maybe(mut self, modifiers_maybe: Option<KeyModifiers>) -> Self {
        if let Some(modifiers) = modifiers_maybe {
            self.modifiers |= modifiers;
        }
        self
    }

    pub fn with_kind(mut self, kind: KeyEventKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_kind_maybe(mut self, kind_maybe: Option<KeyEventKind>) -> Self {
        if let Some(kind) = kind_maybe {
            self.kind = kind;
        }
        self
    }

    pub fn sanitize(mut self) -> Self {
        if self.code == KeyCode::Tab && self.modifiers.contains(KeyModifiers::SHIFT) {
            self.code = KeyCode::BackTab;
        }

        self
    }

    pub fn is_key_pressed(&self) -> bool {
        self.kind == KeyEventKind::Pressed
    }

    pub fn as_key_pressed(&self) -> Option<&KeyEvent> {
        if let KeyEvent {
            code: _,
            modifiers: _,
            kind: KeyEventKind::Pressed,
        } = &self
        {
            return Some(self);
        }

        None
    }

    pub fn is_key_released(&self) -> bool {
        self.kind == KeyEventKind::Released
    }

    pub fn as_key_released(&self) -> Option<&KeyEvent> {
        if let KeyEvent {
            code: _,
            modifiers: _,
            kind: KeyEventKind::Released,
        } = &self
        {
            return Some(self);
        }

        None
    }

    pub fn is_key_repeated(&self) -> bool {
        self.kind == KeyEventKind::Repeated
    }

    pub fn as_key_repeated(&self) -> Option<&KeyEvent> {
        if let KeyEvent {
            code: _,
            modifiers: _,
            kind: KeyEventKind::Repeated,
        } = &self
        {
            return Some(self);
        }

        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Fn(u8),
    Char(char),
    Escape,
    Modifier(ModifierKeyCode),
}

impl KeyCode {
    pub fn is_function_key(&self) -> bool {
        matches!(self, KeyCode::Fn(_))
    }

    pub fn as_function_key(&self) -> Option<&KeyCode> {
        if let KeyCode::Fn(_) = &self {
            return Some(self);
        }

        None
    }

    pub fn is_modifier_key(&self) -> bool {
        matches!(self, KeyCode::Modifier(_))
    }

    pub fn as_modifier_key(&self) -> Option<&KeyCode> {
        if let KeyCode::Modifier(_) = &self {
            return Some(self);
        }

        None
    }

    pub fn is_char_key(&self) -> bool {
        matches!(self, KeyCode::Char(_))
    }

    pub fn as_char_key(&self) -> Option<&KeyCode> {
        if let KeyCode::Char(_) = &self {
            return Some(self);
        }

        None
    }
}

impl From<KeyCode> for KeyEvent {
    fn from(code: KeyCode) -> Self {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Pressed,
        }
    }
}

impl From<(KeyCode, KeyModifiers)> for KeyEvent {
    fn from(value: (KeyCode, KeyModifiers)) -> Self {
        KeyEvent::from(value.0).with_modifiers(value.1)
    }
}

impl From<(KeyCode, Option<KeyModifiers>)> for KeyEvent {
    fn from(value: (KeyCode, Option<KeyModifiers>)) -> Self {
        KeyEvent::from(value.0).with_modifiers_maybe(value.1)
    }
}

impl From<(KeyCode, KeyEventKind)> for KeyEvent {
    fn from(value: (KeyCode, KeyEventKind)) -> Self {
        KeyEvent::from(value.0).with_kind(value.1)
    }
}

impl From<(KeyCode, Option<KeyEventKind>)> for KeyEvent {
    fn from(value: (KeyCode, Option<KeyEventKind>)) -> Self {
        KeyEvent::from(value.0).with_kind_maybe(value.1)
    }
}

impl From<(KeyCode, (KeyModifiers, KeyEventKind))> for KeyEvent {
    fn from(value: (KeyCode, (KeyModifiers, KeyEventKind))) -> Self {
        KeyEvent::from(value.0)
            .with_modifiers(value.1.0)
            .with_kind(value.1.1)
    }
}

impl From<(KeyCode, Option<(KeyModifiers, KeyEventKind)>)> for KeyEvent {
    fn from(value: (KeyCode, Option<(KeyModifiers, KeyEventKind)>)) -> Self {
        if let Some(key_modifiers_and_kind) = value.1 {
            KeyEvent::from((value.0, key_modifiers_and_kind))
        } else {
            KeyEvent::from(value.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModifierKeyCode {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightMeta,
}

impl From<ModifierKeyCode> for KeyEvent {
    fn from(code: ModifierKeyCode) -> Self {
        KeyEvent {
            code: KeyCode::Modifier(code),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Pressed,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 1 << 0;
        const CONTROL = 1 << 1;
        const ALT = 1 << 2;
        const SUPER = 1 << 3;
        const META = 1 << 5;
        const _ = !0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyEventKind {
    Pressed,
    Released,
    Repeated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScreenEvent {
    FocusGained,
    FocusLost,
    Resized { width: u16, height: u16 },
}

impl ScreenEvent {
    pub fn is_focus_gained(&self) -> bool {
        *self == ScreenEvent::FocusGained
    }

    pub fn is_focus_lost(&self) -> bool {
        *self == ScreenEvent::FocusLost
    }

    pub fn is_resized(&self) -> bool {
        matches!(
            self,
            ScreenEvent::Resized {
                width: _,
                height: _
            }
        )
    }

    pub fn as_resized(&self) -> Option<&ScreenEvent> {
        if let ScreenEvent::Resized {
            width: _,
            height: _,
        } = &self
        {
            return Some(self);
        }

        None
    }
}
