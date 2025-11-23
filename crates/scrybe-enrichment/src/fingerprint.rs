//! Fingerprint generation from browser signals.

use blake3::Hasher;
use scrybe_core::{
    types::{Fingerprint, FingerprintComponents, Session},
    ScrybeError,
};
use sha2::{Digest, Sha256};

/// Generates composite fingerprints from browser session data.
pub struct FingerprintGenerator;

impl FingerprintGenerator {
    /// Generate a fingerprint from a session.
    ///
    /// This creates a deterministic composite hash of all browser signals.
    /// Uses SHA-256 for the main hash and BLAKE3 for component hashes.
    ///
    /// # Errors
    ///
    /// Returns `ScrybeError::EnrichmentError` if fingerprint generation fails.
    pub fn generate(session: &Session) -> Result<Fingerprint, ScrybeError> {
        // Generate component hashes
        let components = FingerprintComponents {
            canvas: session.browser.canvas_hash.clone(),
            webgl: session.browser.webgl_hash.clone(),
            audio: session.browser.audio_hash.clone(),
            fonts: Some(Self::hash_fonts(&session.browser.fonts)),
            plugins: Some(Self::hash_plugins(&session.browser.plugins)),
            screen: Some(Self::hash_screen(&session.browser.screen)),
            network: Some(Self::hash_network(&session.network)),
        };

        // Generate composite hash from all components
        let composite_hash = Self::generate_composite_hash(&components);

        // Calculate confidence score based on available signals
        let confidence = Self::calculate_confidence(&components);

        Fingerprint::new(composite_hash, components, confidence as f64)
            .ok_or_else(|| ScrybeError::enrichment_error("fingerprint", "invalid hash generated"))
    }

    /// Generate composite hash from all fingerprint components.
    fn generate_composite_hash(components: &FingerprintComponents) -> String {
        let mut hasher = Sha256::new();

        if let Some(ref canvas) = components.canvas {
            hasher.update(canvas.as_bytes());
        }
        if let Some(ref webgl) = components.webgl {
            hasher.update(webgl.as_bytes());
        }
        if let Some(ref audio) = components.audio {
            hasher.update(audio.as_bytes());
        }
        if let Some(ref fonts) = components.fonts {
            hasher.update(fonts.as_bytes());
        }
        if let Some(ref plugins) = components.plugins {
            hasher.update(plugins.as_bytes());
        }
        if let Some(ref screen) = components.screen {
            hasher.update(screen.as_bytes());
        }
        if let Some(ref network) = components.network {
            hasher.update(network.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Hash font list using BLAKE3.
    fn hash_fonts(fonts: &[String]) -> String {
        let mut hasher = Hasher::new();
        for font in fonts {
            hasher.update(font.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Hash plugin list using BLAKE3.
    fn hash_plugins(plugins: &[String]) -> String {
        let mut hasher = Hasher::new();
        for plugin in plugins {
            hasher.update(plugin.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Hash screen info using BLAKE3.
    fn hash_screen(screen: &scrybe_core::types::ScreenInfo) -> String {
        let mut hasher = Hasher::new();
        hasher.update(&screen.width.to_le_bytes());
        hasher.update(&screen.height.to_le_bytes());
        hasher.update(&screen.color_depth.to_le_bytes());
        hasher.update(&screen.pixel_ratio.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// Hash network signals using BLAKE3.
    fn hash_network(network: &scrybe_core::types::NetworkSignals) -> String {
        let mut hasher = Hasher::new();
        hasher.update(network.ip.to_string().as_bytes());
        if let Some(ref ja3) = network.ja3 {
            hasher.update(ja3.as_bytes());
        }
        if let Some(ref ja4) = network.ja4 {
            hasher.update(ja4.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Calculate confidence score based on available signals.
    ///
    /// Score ranges from 0.0 (low confidence) to 1.0 (high confidence).
    fn calculate_confidence(components: &FingerprintComponents) -> f32 {
        let mut signal_count = 0;
        let mut total_weight = 0.0;

        // Canvas fingerprint (weight: 0.25)
        if components.canvas.is_some() {
            signal_count += 1;
            total_weight += 0.25;
        }

        // WebGL fingerprint (weight: 0.25)
        if components.webgl.is_some() {
            signal_count += 1;
            total_weight += 0.25;
        }

        // Audio fingerprint (weight: 0.15)
        if components.audio.is_some() {
            signal_count += 1;
            total_weight += 0.15;
        }

        // Fonts (weight: 0.15)
        if components.fonts.is_some() {
            signal_count += 1;
            total_weight += 0.15;
        }

        // Plugins (weight: 0.10)
        if components.plugins.is_some() {
            signal_count += 1;
            total_weight += 0.10;
        }

        // Screen (weight: 0.05)
        if components.screen.is_some() {
            signal_count += 1;
            total_weight += 0.05;
        }

        // Network (weight: 0.05)
        if components.network.is_some() {
            signal_count += 1;
            total_weight += 0.05;
        }

        // Normalize to 0.0-1.0 range
        if signal_count == 0 {
            0.0
        } else {
            total_weight
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fingerprint_generation() {
        // Test will be implemented when we have full session creation
        // For now, just ensure the module compiles
        assert!(true);
    }
}
