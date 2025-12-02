//! Input management system for keyboard and mouse.
//!
//! Provides generic input tracking with pressed/just_pressed/just_released states.
//!
//! # Examples
//!
//! ```
//! use aaa_ecs::builtin::{KeyboardInput, KeyCode};
//!
//! let mut input = KeyboardInput::new();
//! input.press(KeyCode::Space);
//!
//! if input.just_pressed(KeyCode::Space) {
//!     println!("Jump!");
//! }
//!
//! input.clear_just_changed(); // Call at end of frame
//! ```

use std::collections::HashSet;
use std::hash::Hash;

/// Generic input tracker
#[derive(Clone, Debug)]
pub struct Input<T: Copy + Eq + Hash> {
    pressed: HashSet<T>,
    just_pressed: HashSet<T>,
    just_released: HashSet<T>,
}

impl<T: Copy + Eq + Hash> Input<T> {
    /// Create new input tracker
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    /// Press a key/button
    pub fn press(&mut self, input: T) {
        if !self.pressed.contains(&input) {
            self.just_pressed.insert(input);
        }
        self.pressed.insert(input);
    }

    /// Release a key/button
    pub fn release(&mut self, input: T) {
        if self.pressed.contains(&input) {
            self.just_released.insert(input);
        }
        self.pressed.remove(&input);
    }

    /// Check if input is currently pressed
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    /// Check if input was just pressed this frame
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Check if input was just released this frame
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    /// Get all currently pressed inputs
    pub fn get_pressed(&self) -> impl Iterator<Item = &T> {
        self.pressed.iter()
    }

    /// Clear just_pressed and just_released (call at end of frame)
    pub fn clear_just_changed(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Reset all input state
    pub fn reset(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

impl<T: Copy + Eq + Hash> Default for Input<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard key codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Special keys
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,

    // Arrow keys
    Left,
    Right,
    Up,
    Down,

    // Modifiers
    LShift,
    RShift,
    LControl,
    RControl,
    LAlt,
    RAlt,
}

/// Mouse button codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Type alias for keyboard input
pub type KeyboardInput = Input<KeyCode>;

/// Type alias for mouse button input
pub type MouseInput = Input<MouseButton>;

/// Mouse position and delta
#[derive(Clone, Copy, Debug, Default)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
}

impl MousePosition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, x: f32, y: f32) {
        self.delta_x = x - self.x;
        self.delta_y = y - self.y;
        self.x = x;
        self.y = y;
    }

    pub fn clear_delta(&mut self) {
        self.delta_x = 0.0;
        self.delta_y = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_press_release() {
        let mut input = Input::<KeyCode>::new();

        // Press key
        input.press(KeyCode::Space);
        assert!(input.pressed(KeyCode::Space));
        assert!(input.just_pressed(KeyCode::Space));

        // Clear just_pressed
        input.clear_just_changed();
        assert!(input.pressed(KeyCode::Space));
        assert!(!input.just_pressed(KeyCode::Space));

        // Release key
        input.release(KeyCode::Space);
        assert!(!input.pressed(KeyCode::Space));
        assert!(input.just_released(KeyCode::Space));
    }

    #[test]
    fn test_mouse_position() {
        let mut pos = MousePosition::new();

        pos.update(10.0, 20.0);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);

        pos.update(15.0, 25.0);
        assert_eq!(pos.delta_x, 5.0);
        assert_eq!(pos.delta_y, 5.0);
    }
}
