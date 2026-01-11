/// Configuration constants for the posture detection application

// Model and image processing configuration
pub const MOVENET_SIZE: u32 = 256;
pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;
pub const GOOD_POSTURE_DEVIATION: f32 = 10.0; // Sensitivity
pub const CAMERA_ROTATION_DEGREES: u32 = 180; // 0, 90, 180, or 270 degrees

// Debounce and fade settings
pub const DEBOUNCE_FRAMES: usize = 15; // How many bad frames before trigger?
pub const MAX_ALPHA: u32 = 180;        // Max opacity (0-255)
pub const FADE_SPEED: u32 = 15;        // How fast it fades in/out
