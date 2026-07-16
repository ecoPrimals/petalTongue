// SPDX-License-Identifier: AGPL-3.0-or-later

/// Classification of gesture events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureType {
    /// Swipe in a direction.
    Swipe(GestureDirection),
    /// Pinch (zoom in).
    PinchIn,
    /// Spread (zoom out).
    PinchOut,
    /// Rotation gesture.
    Rotate,
    /// Wave / attention-getting gesture.
    Wave,
    /// Point at target.
    Point,
    /// Grab / grip.
    Grab,
    /// Release / open hand.
    Release,
    /// Custom gesture with a name.
    Custom(String),
}

/// Direction for swipe/directional gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureDirection {
    /// Swipe up.
    Up,
    /// Swipe down.
    Down,
    /// Swipe left.
    Left,
    /// Swipe right.
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gesture_type_equality() {
        assert_eq!(GestureType::Wave, GestureType::Wave);
        assert_eq!(
            GestureType::Swipe(GestureDirection::Up),
            GestureType::Swipe(GestureDirection::Up)
        );
        assert_ne!(
            GestureType::Swipe(GestureDirection::Up),
            GestureType::Swipe(GestureDirection::Down)
        );
        assert_ne!(GestureType::PinchIn, GestureType::PinchOut);
    }

    #[test]
    fn gesture_direction_equality() {
        assert_eq!(GestureDirection::Left, GestureDirection::Left);
        assert_ne!(GestureDirection::Left, GestureDirection::Right);
    }
}
