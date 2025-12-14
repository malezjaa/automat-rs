use crate::{Action, Result, with_enigo};
use enigo::{Axis, Button, Coordinate, Direction, Mouse};

/// Performs mouse operations including movement, clicks, and scrolling.
pub struct MouseAction {
  kind: MouseActionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseActionKind {
  Move {
    x: i32,
    y: i32,
    coordinate: Coordinate,
  },
  Click {
    button: Button,
    direction: Direction,
  },
  Scroll {
    axis: Axis,
    length: i32,
  },
}

impl MouseAction {
  /// Creates a mouse movement to absolute screen coordinates.
  pub fn move_mouse(x: i32, y: i32) -> Self {
    Self::move_mouse_with_coordinate(x, y, Coordinate::Abs)
  }

  /// Create a mouse movement using relative screen coordinates.
  pub fn move_mouse_relative(x: i32, y: i32) -> Self {
    Self::move_mouse_with_coordinate(x, y, Coordinate::Rel)
  }

  /// Creates a mouse movement with a specified coordinate system.
  fn move_mouse_with_coordinate(x: i32, y: i32, coordinate: Coordinate) -> Self {
    Self {
      kind: MouseActionKind::Move { x, y, coordinate },
    }
  }

  /// Creates a full mouse click (press and release).
  #[inline]
  pub fn click(button: Button) -> Self {
    Self::with_button_direction(button, Direction::Click)
  }

  /// Creates a mouse button press.
  #[inline]
  pub fn press(button: Button) -> Self {
    Self::with_button_direction(button, Direction::Press)
  }

  /// Creates a mouse button release.
  #[inline]
  pub fn release(button: Button) -> Self {
    Self::with_button_direction(button, Direction::Release)
  }

  /// Helper to create click actions with a specific direction.
  fn with_button_direction(button: Button, direction: Direction) -> Self {
    Self {
      kind: MouseActionKind::Click { button, direction },
    }
  }

  /// Changes the direction for click actions.
  pub fn with_direction(mut self, direction: Direction) -> Self {
    if let MouseActionKind::Click { direction: d, .. } = &mut self.kind {
      *d = direction;
    }
    self
  }

  /// Creates a scroll action on the specified axis.
  pub fn scroll(length: i32, axis: Axis) -> Self {
    Self {
      kind: MouseActionKind::Scroll { axis, length },
    }
  }

  /// Returns the underlying action kind.
  #[inline]
  pub const fn kind(&self) -> &MouseActionKind {
    &self.kind
  }
}

impl Action for MouseAction {
  fn run(&self) -> Result<()> {
    with_enigo(|e| match self.kind() {
      MouseActionKind::Move { x, y, coordinate } => e.move_mouse(*x, *y, *coordinate),
      MouseActionKind::Click { button, direction } => e.button(*button, *direction),
      MouseActionKind::Scroll { axis, length } => e.scroll(*length, *axis),
    })
    .map_err(Into::into)
  }
}
