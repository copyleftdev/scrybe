//! Browser environment signals.

use serde::{Deserialize, Serialize};

/// Browser environment signals collected from browser APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrowserSignals {
    /// Canvas fingerprint hash (SHA-256)
    pub canvas_hash: Option<String>,
    /// WebGL fingerprint hash (SHA-256)
    pub webgl_hash: Option<String>,
    /// Audio fingerprint hash (SHA-256)
    pub audio_hash: Option<String>,
    /// List of installed fonts
    pub fonts: Vec<String>,
    /// List of browser plugins
    pub plugins: Vec<String>,
    /// Timezone (IANA timezone identifier)
    pub timezone: String,
    /// Browser language
    pub language: String,
    /// Screen information
    pub screen: ScreenInfo,
    /// User agent string
    pub user_agent: String,
}

/// Screen and display information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenInfo {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Available width (excluding taskbars, etc.)
    pub avail_width: u32,
    /// Available height (excluding taskbars, etc.)
    pub avail_height: u32,
    /// Color depth in bits
    pub color_depth: u8,
    /// Pixel ratio (e.g., 2.0 for Retina displays)
    pub pixel_ratio: f32,
}

impl ScreenInfo {
    /// Create new screen info with validation.
    ///
    /// # Errors
    ///
    /// Returns `None` if dimensions are zero or unrealistic.
    pub fn new(
        width: u32,
        height: u32,
        avail_width: u32,
        avail_height: u32,
        color_depth: u8,
        pixel_ratio: f32,
    ) -> Option<Self> {
        // Validate dimensions
        if width == 0 || height == 0 || width > 10000 || height > 10000 {
            return None;
        }

        // Validate color depth (typical values: 8, 16, 24, 32)
        if color_depth == 0 || color_depth > 48 {
            return None;
        }

        // Validate pixel ratio
        if pixel_ratio <= 0.0 || pixel_ratio > 5.0 {
            return None;
        }

        Some(Self {
            width,
            height,
            avail_width,
            avail_height,
            color_depth,
            pixel_ratio,
        })
    }
}

impl Default for ScreenInfo {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            avail_width: 1920,
            avail_height: 1080,
            color_depth: 24,
            pixel_ratio: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_info_validation() {
        // Valid screen info
        let screen = ScreenInfo::new(1920, 1080, 1920, 1080, 24, 1.0);
        assert!(screen.is_some());

        // Zero width
        let screen = ScreenInfo::new(0, 1080, 0, 1080, 24, 1.0);
        assert!(screen.is_none());

        // Unrealistic dimensions
        let screen = ScreenInfo::new(20000, 1080, 20000, 1080, 24, 1.0);
        assert!(screen.is_none());

        // Invalid color depth
        let screen = ScreenInfo::new(1920, 1080, 1920, 1080, 0, 1.0);
        assert!(screen.is_none());

        // Invalid pixel ratio
        let screen = ScreenInfo::new(1920, 1080, 1920, 1080, 24, 0.0);
        assert!(screen.is_none());
    }

    #[test]
    fn test_screen_info_default() {
        let screen = ScreenInfo::default();
        assert_eq!(screen.width, 1920);
        assert_eq!(screen.height, 1080);
        assert_eq!(screen.color_depth, 24);
    }

    #[test]
    fn test_browser_signals_serialization() {
        let signals = BrowserSignals {
            canvas_hash: Some("abc123".to_string()),
            webgl_hash: None,
            audio_hash: None,
            fonts: vec!["Arial".to_string(), "Helvetica".to_string()],
            plugins: vec![],
            timezone: "America/New_York".to_string(),
            language: "en-US".to_string(),
            screen: ScreenInfo::default(),
            user_agent: "Mozilla/5.0".to_string(),
        };

        let json = serde_json::to_string(&signals).unwrap();
        let deserialized: BrowserSignals = serde_json::from_str(&json).unwrap();
        assert_eq!(signals, deserialized);
    }

    #[test]
    fn test_retina_display() {
        let screen = ScreenInfo::new(2880, 1800, 2880, 1800, 24, 2.0);
        assert!(screen.is_some());
        let screen = screen.unwrap();
        assert_eq!(screen.pixel_ratio, 2.0);
    }
}
