use crate::actions::with_enigo;
use crate::{Action, Result};
use enigo::{Direction, Key, Keyboard};

/// A keyboard action that can either type text or press/release a key.
pub struct KeyboardAction(KeyboardActionKind);

/// The kind of keyboard action to perform.
pub enum KeyboardActionKind {
    /// Type a string of text.
    Text(String),
    /// Press or release a specific key.
    Key(Key, Direction),
}

impl KeyboardAction {
    /// Creates a new keyboard action that types the given text.
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self(KeyboardActionKind::Text(text.into()))
    }

    /// Creates a new keyboard action that presses or releases a key.
    pub fn key(key: Key, direction: Direction) -> Self {
        Self(KeyboardActionKind::Key(key, direction))
    }

    /// Returns the kind of keyboard action.
    pub fn kind(&self) -> &KeyboardActionKind {
        &self.0
    }
}

impl Action for KeyboardAction {
    fn run(&self) -> Result<()> {
        with_enigo(|e| match self.kind() {
            KeyboardActionKind::Text(text) => e.text(&text),
            KeyboardActionKind::Key(key, direction) => e.key(*key, *direction),
        })
        .map_err(Into::into)
    }
}
