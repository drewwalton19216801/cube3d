use druid::Data;

/// Application state
#[derive(Clone, Data)]
pub struct AppState {
    /// Current rotation angle around the X-axis
    pub angle_x: f64,
    /// Current rotation angle around the Y-axis
    pub angle_y: f64,
    /// Translation vector (x, y)
    pub translation: [f64; 2],
    /// Enable debug mode
    pub debug: bool,
    /// Simulation paused
    pub paused: bool,
    /// Wireframe mode enabled
    pub wireframe: bool,
    /// Zoom level
    pub zoom: f64,
    /// Light position in world space
    pub light_position: [f64; 3],
}
