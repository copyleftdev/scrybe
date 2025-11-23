//! User behavioral patterns and timing metrics.

use serde::{Deserialize, Serialize};

/// User behavioral patterns collected during session.
///
/// All event collections are bounded to prevent DoS attacks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BehavioralSignals {
    /// Mouse movement events (max 1000)
    pub mouse_events: Vec<MouseEvent>,
    /// Scroll events (max 100)
    pub scroll_events: Vec<ScrollEvent>,
    /// Click events (max 100)
    pub click_events: Vec<ClickEvent>,
    /// Timing metrics
    pub timing: TimingMetrics,
}

/// Mouse event (movement or click).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MouseEvent {
    /// Timestamp in milliseconds since session start
    pub timestamp_ms: u64,
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Event type
    pub event_type: MouseEventType,
}

/// Type of mouse event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseEventType {
    /// Mouse movement
    Move,
    /// Mouse click
    Click,
    /// Mouse down
    Down,
    /// Mouse up
    Up,
}

/// Scroll event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrollEvent {
    /// Timestamp in milliseconds since session start
    pub timestamp_ms: u64,
    /// Scroll X position
    pub x: i32,
    /// Scroll Y position
    pub y: i32,
    /// Scroll delta X
    pub delta_x: i32,
    /// Scroll delta Y
    pub delta_y: i32,
}

/// Click event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClickEvent {
    /// Timestamp in milliseconds since session start
    pub timestamp_ms: u64,
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Mouse button
    pub button: MouseButton,
}

/// Mouse button identifier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button (wheel)
    Middle,
    /// Right mouse button
    Right,
    /// Other/auxiliary button
    Other(u8),
}

/// Timing metrics for page load and interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TimingMetrics {
    /// DOM content loaded time (ms)
    pub dom_content_loaded_ms: Option<u64>,
    /// Full page load time (ms)
    pub load_time_ms: Option<u64>,
    /// Time to first byte (ms)
    pub time_to_first_byte_ms: Option<u64>,
    /// Time to first interaction (ms)
    pub time_to_first_interaction_ms: Option<u64>,
}

/// Maximum number of mouse events to store (DoS protection).
pub const MAX_MOUSE_EVENTS: usize = 1000;

/// Maximum number of scroll events to store (DoS protection).
pub const MAX_SCROLL_EVENTS: usize = 100;

/// Maximum number of click events to store (DoS protection).
pub const MAX_CLICK_EVENTS: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_event_creation() {
        let event = MouseEvent {
            timestamp_ms: 100,
            x: 50,
            y: 75,
            event_type: MouseEventType::Move,
        };

        assert_eq!(event.timestamp_ms, 100);
        assert_eq!(event.x, 50);
        assert_eq!(event.y, 75);
        assert_eq!(event.event_type, MouseEventType::Move);
    }

    #[test]
    fn test_click_event_with_button() {
        let event = ClickEvent {
            timestamp_ms: 200,
            x: 100,
            y: 150,
            button: MouseButton::Left,
        };

        assert_eq!(event.button, MouseButton::Left);
    }

    #[test]
    fn test_timing_metrics_default() {
        let timing = TimingMetrics::default();
        assert!(timing.dom_content_loaded_ms.is_none());
        assert!(timing.load_time_ms.is_none());
    }

    #[test]
    fn test_behavioral_signals_serialization() {
        let signals = BehavioralSignals {
            mouse_events: vec![MouseEvent {
                timestamp_ms: 100,
                x: 10,
                y: 20,
                event_type: MouseEventType::Click,
            }],
            scroll_events: vec![],
            click_events: vec![],
            timing: TimingMetrics::default(),
        };

        let json = serde_json::to_string(&signals).unwrap();
        let deserialized: BehavioralSignals = serde_json::from_str(&json).unwrap();
        assert_eq!(signals, deserialized);
    }

    #[test]
    fn test_bounded_collection_constants() {
        // Verify DoS protection limits are reasonable
        assert!(MAX_MOUSE_EVENTS <= 1000);
        assert!(MAX_SCROLL_EVENTS <= 100);
        assert!(MAX_CLICK_EVENTS <= 100);
    }
}
