//! Fingerprint Protection
//!
//! Protects against browser fingerprinting by randomizing or blocking
//! APIs commonly used to create unique device fingerprints.
//!
//! ## Techniques
//!
//! 1. **Canvas Fingerprinting**: Add subtle noise to canvas readback
//! 2. **WebGL Fingerprinting**: Randomize renderer/vendor strings
//! 3. **Audio Fingerprinting**: Add noise to AudioContext output
//! 4. **Font Fingerprinting**: Limit font enumeration
//! 5. **Screen Fingerprinting**: Round screen dimensions
//! 6. **Hardware Fingerprinting**: Limit navigator properties

use super::ShieldLevel;

/// Fingerprint protection configuration
pub struct FingerprintProtection {
    /// Seed for consistent randomization (per session)
    seed: u64,
}

impl FingerprintProtection {
    pub fn new() -> Self {
        // Generate a random seed for this session
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);

        Self { seed }
    }

    /// Get the JavaScript protection script
    pub fn get_protection_script(&self, level: ShieldLevel) -> String {
        match level {
            ShieldLevel::Off => String::new(),
            ShieldLevel::Standard => self.standard_protection_script(),
            ShieldLevel::Strict => self.strict_protection_script(),
        }
    }

    /// Standard protection - minimal site breakage
    fn standard_protection_script(&self) -> String {
        format!(r#"
(function() {{
    'use strict';
    
    const SHIELD_SEED = {};
    
    // Simple seeded random for consistent noise
    function seededRandom(seed) {{
        const x = Math.sin(seed) * 10000;
        return x - Math.floor(x);
    }}
    
    // ===== Canvas Protection =====
    // Add subtle noise to canvas readback to prevent fingerprinting
    const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
    const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;
    
    HTMLCanvasElement.prototype.toDataURL = function(...args) {{
        const ctx = this.getContext('2d');
        if (ctx && this.width > 16 && this.height > 16) {{
            // Add very subtle noise that's invisible to humans
            const imageData = ctx.getImageData(0, 0, this.width, this.height);
            const data = imageData.data;
            for (let i = 0; i < data.length; i += 4) {{
                // Only modify pixels very slightly (±1)
                const noise = Math.floor(seededRandom(SHIELD_SEED + i) * 3) - 1;
                data[i] = Math.max(0, Math.min(255, data[i] + noise));
            }}
            ctx.putImageData(imageData, 0, 0);
        }}
        return originalToDataURL.apply(this, args);
    }};
    
    // ===== WebGL Protection =====
    // Randomize WebGL renderer strings
    const getParameterProto = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(param) {{
        // UNMASKED_VENDOR_WEBGL
        if (param === 37445) {{
            return 'Spacey Graphics';
        }}
        // UNMASKED_RENDERER_WEBGL
        if (param === 37446) {{
            return 'Spacey WebGL Renderer';
        }}
        return getParameterProto.call(this, param);
    }};
    
    // WebGL2 too
    if (typeof WebGL2RenderingContext !== 'undefined') {{
        const getParameter2Proto = WebGL2RenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getParameter = function(param) {{
            if (param === 37445) return 'Spacey Graphics';
            if (param === 37446) return 'Spacey WebGL2 Renderer';
            return getParameter2Proto.call(this, param);
        }};
    }}
    
    // ===== Navigator Protection =====
    // Limit hardware concurrency exposure
    Object.defineProperty(navigator, 'hardwareConcurrency', {{
        get: function() {{ return 4; }}, // Report common value
        configurable: false
    }});
    
    // Round device memory
    if (navigator.deviceMemory) {{
        Object.defineProperty(navigator, 'deviceMemory', {{
            get: function() {{ return 8; }}, // Report common value
            configurable: false
        }});
    }}
    
    // ===== Screen Protection =====
    // Round screen dimensions to common values
    const roundToCommon = (val) => {{
        const common = [768, 800, 900, 1024, 1050, 1080, 1200, 1440, 1600, 2160];
        return common.reduce((a, b) => Math.abs(b - val) < Math.abs(a - val) ? b : a);
    }};
    
    Object.defineProperty(screen, 'width', {{
        get: function() {{ return roundToCommon(screen.width); }},
        configurable: false
    }});
    
    Object.defineProperty(screen, 'height', {{
        get: function() {{ return roundToCommon(screen.height); }},
        configurable: false
    }});
    
    console.log('[Spacey Shield] Fingerprint protection active (Standard)');
}})();
"#, self.seed)
    }

    /// Strict protection - more aggressive, may break some sites
    fn strict_protection_script(&self) -> String {
        let standard = self.standard_protection_script();
        
        format!(r#"
{}

(function() {{
    'use strict';
    
    const SHIELD_SEED = {};
    
    // ===== Audio Fingerprinting Protection =====
    // Add noise to AudioContext
    if (typeof AudioContext !== 'undefined') {{
        const originalCreateOscillator = AudioContext.prototype.createOscillator;
        const originalCreateDynamicsCompressor = AudioContext.prototype.createDynamicsCompressor;
        
        AudioContext.prototype.createOscillator = function() {{
            const osc = originalCreateOscillator.call(this);
            // Slightly detune to prevent fingerprinting
            osc.detune.value = (Math.random() - 0.5) * 0.001;
            return osc;
        }};
    }}
    
    // ===== Font Fingerprinting Protection =====
    // Limit font detection by overriding measureText
    const originalMeasureText = CanvasRenderingContext2D.prototype.measureText;
    CanvasRenderingContext2D.prototype.measureText = function(text) {{
        const result = originalMeasureText.call(this, text);
        // Round width to prevent precise font detection
        const originalWidth = result.width;
        Object.defineProperty(result, 'width', {{
            get: function() {{ return Math.round(originalWidth); }},
            configurable: false
        }});
        return result;
    }};
    
    // ===== Client Rects Protection =====
    // Randomize element positioning slightly
    const originalGetBoundingClientRect = Element.prototype.getBoundingClientRect;
    Element.prototype.getBoundingClientRect = function() {{
        const rect = originalGetBoundingClientRect.call(this);
        const noise = 0.00001;
        return {{
            x: rect.x + noise,
            y: rect.y + noise,
            width: rect.width,
            height: rect.height,
            top: rect.top + noise,
            right: rect.right + noise,
            bottom: rect.bottom + noise,
            left: rect.left + noise,
            toJSON: rect.toJSON
        }};
    }};
    
    // ===== Battery API =====
    // Block battery API (major fingerprinting vector)
    if (navigator.getBattery) {{
        navigator.getBattery = function() {{
            return Promise.reject(new Error('Battery API disabled for privacy'));
        }};
    }}
    
    // ===== Gamepad API =====
    // Block gamepad API
    navigator.getGamepads = function() {{
        return [];
    }};
    
    // ===== Keyboard/Language Fingerprinting =====
    Object.defineProperty(navigator, 'keyboard', {{
        get: function() {{ return undefined; }},
        configurable: false
    }});
    
    Object.defineProperty(navigator, 'languages', {{
        get: function() {{ return ['en-US', 'en']; }}, // Common value
        configurable: false
    }});
    
    // ===== Connection API =====
    // Limit network information exposure
    if (navigator.connection) {{
        Object.defineProperty(navigator, 'connection', {{
            get: function() {{ return undefined; }},
            configurable: false
        }});
    }}
    
    // ===== Media Devices =====
    // Limit device enumeration
    if (navigator.mediaDevices && navigator.mediaDevices.enumerateDevices) {{
        const originalEnumerateDevices = navigator.mediaDevices.enumerateDevices;
        navigator.mediaDevices.enumerateDevices = function() {{
            return originalEnumerateDevices.call(this).then(devices => {{
                // Return generic device info without unique IDs
                return devices.map(device => ({{
                    deviceId: '',
                    groupId: '',
                    kind: device.kind,
                    label: ''
                }}));
            }});
        }};
    }}
    
    console.log('[Spacey Shield] Fingerprint protection active (Strict)');
}})();
"#, standard, self.seed)
    }
}

impl Default for FingerprintProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_generation() {
        let fp = FingerprintProtection::new();
        
        let standard = fp.get_protection_script(ShieldLevel::Standard);
        assert!(standard.contains("Canvas Protection"));
        assert!(standard.contains("WebGL Protection"));
        
        let strict = fp.get_protection_script(ShieldLevel::Strict);
        assert!(strict.contains("Audio Fingerprinting"));
        assert!(strict.contains("Battery API"));
    }

    #[test]
    fn test_off_returns_empty() {
        let fp = FingerprintProtection::new();
        let script = fp.get_protection_script(ShieldLevel::Off);
        assert!(script.is_empty());
    }
}
