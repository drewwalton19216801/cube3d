mod state;
mod widget;
mod vertex;
mod math;
mod graphics;

use druid::{AppLauncher, PlatformError, WindowDesc, LocalizedString};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(widget::CubeWidget::new())
        .title(LocalizedString::new("3D Cube with Per-Pixel Lighting"))
        .window_size((400.0, 400.0));

    let initial_state = state::AppState {
        angle_x: 0.0,
        angle_y: 0.0,
        translation: [0.0, 0.0], // Initialize translation
        debug: false,
        paused: false,
        wireframe: false,
        zoom: 1.0, // Initialize zoom level
    };

    AppLauncher::with_window(main_window).launch(initial_state)?;

    Ok(())
}
