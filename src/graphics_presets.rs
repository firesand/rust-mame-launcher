use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VideoBackend {
    Default,
    OpenGL,
    BGFX,
    Software,
    D3D,     // Windows only
    Metal,   // macOS only
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsPreset {
    pub name: String,
    pub description: String,
    pub video_backend: VideoBackend,
    pub shader_chain: Option<String>,
    pub filter: bool,
    pub prescale: u8,
    pub scanlines: bool,
    pub keep_aspect: bool,
    pub custom_args: Vec<String>,
}

impl GraphicsPreset {
    /// Get built-in presets
    pub fn get_defaults() -> Vec<GraphicsPreset> {
        vec![
            GraphicsPreset {
                name: "Original".to_string(),
                description: "Raw pixels, no enhancements".to_string(),
                video_backend: VideoBackend::Default,
                shader_chain: None,
                filter: false,
                prescale: 1,
                scanlines: false,
                keep_aspect: true,
                custom_args: vec![],
            },
            GraphicsPreset {
                name: "CRT Classic".to_string(),
                description: "Arcade monitor with scanlines and curvature".to_string(),
                video_backend: VideoBackend::BGFX,
                shader_chain: Some("crt-geom".to_string()),
                filter: true,
                prescale: 1,
                scanlines: true,
                keep_aspect: true,
                custom_args: vec![],
            },
            GraphicsPreset {
                name: "CRT Deluxe".to_string(),
                description: "Enhanced CRT with bloom and halation".to_string(),
                video_backend: VideoBackend::BGFX,
                shader_chain: Some("crt-geom-deluxe".to_string()),
                filter: true,
                prescale: 1,
                scanlines: true,
                keep_aspect: true,
                custom_args: vec!["-bloom_lvl".to_string(), "1.5".to_string()],
            },
            GraphicsPreset {
                name: "Sharp Pixels".to_string(),
                description: "Integer scaled, no filtering".to_string(),
                video_backend: VideoBackend::OpenGL,
                shader_chain: None,
                filter: false,
                prescale: 3,
                scanlines: false,
                keep_aspect: true,
                custom_args: vec!["-nounevenstretch".to_string()],
            },
            GraphicsPreset {
                name: "Smooth HD".to_string(),
                description: "Bilinear filtering for smooth appearance".to_string(),
                video_backend: VideoBackend::OpenGL,
                shader_chain: None,
                filter: true,
                prescale: 2,
                scanlines: false,
                keep_aspect: true,
                custom_args: vec![],
            },
            GraphicsPreset {
                name: "LCD Grid".to_string(),
                description: "Modern LCD/LED display simulation".to_string(),
                video_backend: VideoBackend::BGFX,
                shader_chain: Some("lcd-grid".to_string()),
                filter: true,
                prescale: 1,
                scanlines: false,
                keep_aspect: true,
                custom_args: vec![],
            },
            GraphicsPreset {
                name: "Arcade Phosphor".to_string(),
                description: "Phosphor glow and aperture grille".to_string(),
                video_backend: VideoBackend::BGFX,
                shader_chain: Some("crt-geom,aperture,bloom".to_string()),
                filter: true,
                prescale: 1,
                scanlines: true,
                keep_aspect: true,
                custom_args: vec![],
            },
        ]
    }

    /// Build MAME command line arguments
    pub fn to_mame_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Video backend
        match &self.video_backend {
            VideoBackend::OpenGL => args.extend(["-video".to_string(), "opengl".to_string()]),
            VideoBackend::BGFX => args.extend(["-video".to_string(), "bgfx".to_string()]),
            VideoBackend::Software => args.extend(["-video".to_string(), "soft".to_string()]),
            VideoBackend::D3D => args.extend(["-video".to_string(), "d3d".to_string()]),
            VideoBackend::Metal => args.extend(["-video".to_string(), "metal".to_string()]),
            VideoBackend::Default => {},
        }

        // Shader chain
        if let Some(chain) = &self.shader_chain {
            if self.video_backend == VideoBackend::BGFX {
                args.extend(["-bgfx_screen_chains".to_string(), chain.clone()]);
            } else if self.video_backend == VideoBackend::OpenGL {
                args.extend(["-gl_glsl".to_string(), "1".to_string()]);
                args.extend(["-glsl_shader_mame0".to_string(), chain.clone()]);
            }
        }

        // Filtering
        if self.filter {
            args.push("-filter".to_string());
        } else {
            args.push("-nofilter".to_string());
        }

        // Prescale
        if self.prescale > 1 {
            args.extend(["-prescale".to_string(), self.prescale.to_string()]);
        }

        // Aspect ratio
        if self.keep_aspect {
            args.push("-keepaspect".to_string());
        }

        // Custom arguments
        args.extend(self.custom_args.clone());

        args
    }
}

/// Per-game graphics overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameGraphicsOverride {
    pub rom_name: String,
    pub preset_name: String,
    pub custom_args: Vec<String>,
}

/// Graphics configuration manager
#[derive(Debug, Clone, Serialize, Deserialize)]  // ← ADD THIS LINE
pub struct GraphicsConfig {
    pub presets: Vec<GraphicsPreset>,
    pub custom_presets: Vec<GraphicsPreset>,
    pub game_overrides: HashMap<String, GameGraphicsOverride>,
    pub global_preset: String,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            presets: GraphicsPreset::get_defaults(),
            custom_presets: vec![],
            game_overrides: HashMap::new(),
            global_preset: "Original".to_string(),
        }
    }
}

impl GraphicsConfig {
    /// Get preset by name
    pub fn get_preset(&self, name: &str) -> Option<&GraphicsPreset> {
        self.presets.iter()
        .chain(self.custom_presets.iter())
        .find(|p| p.name == name)
    }

    /// Get preset for a specific game
    pub fn get_game_preset(&self, rom_name: &str) -> &GraphicsPreset {
        if let Some(override_cfg) = self.game_overrides.get(rom_name) {
            if let Some(preset) = self.get_preset(&override_cfg.preset_name) {
                return preset;
            }
        }

        self.get_preset(&self.global_preset)
        .unwrap_or(&self.presets[0])
    }

    /// Create a custom preset
    pub fn add_custom_preset(&mut self, preset: GraphicsPreset) {
        self.custom_presets.push(preset);
    }
}
