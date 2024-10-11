use druid::kurbo::Point;
use druid::text::FontFamily;
use druid::widget::prelude::*;
use druid::{
    commands,
    piet::{InterpolationMode, Text, TextLayout, TextLayoutBuilder},
    AppLauncher, Color, Data, LocalizedString, PlatformError, RenderContext, Widget, WindowDesc,
};
use std::time::Instant;

/// Application state
#[derive(Clone, Data)]
struct AppState {
    /// Current rotation angle around the X-axis
    angle_x: f64,
    /// Current rotation angle around the Y-axis
    angle_y: f64,
    /// Translation vector (x, y)
    translation: [f64; 2],
    /// Enable debug mode
    debug: bool,
    /// Simulation paused
    paused: bool,
    /// Wireframe mode enabled
    wireframe: bool,
    /// Zoom level
    zoom: f64,
}

/// 3D cube widget
struct CubeWidget {
    frames_since_last_update: usize,
    last_fps_calculation: Instant,
    fps: f64,
    /// Is the user currently dragging for rotation?
    dragging_rotation: bool,
    /// Is the user currently dragging for translation?
    dragging_translation: bool,
    /// Last mouse position
    last_mouse_pos: Point,
}

impl CubeWidget {
    fn new() -> Self {
        CubeWidget {
            frames_since_last_update: 0,
            last_fps_calculation: Instant::now(),
            fps: 0.0,
            dragging_rotation: false,
            dragging_translation: false,
            last_mouse_pos: Point::ZERO,
        }
    }
}

impl Widget<AppState> for CubeWidget {
    /// Handle events for the cube widget
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_timer(std::time::Duration::from_millis(16));
                // Request focus to receive keyboard events
                ctx.request_focus();
            }
            Event::Timer(_) => {
                if !data.paused && !self.dragging_rotation && !self.dragging_translation {
                    data.angle_x += 0.01;
                    data.angle_y += 0.02;
                    ctx.request_paint();
                }
                ctx.request_timer(std::time::Duration::from_millis(16));
            }
            Event::KeyDown(key_event) => {
                if let druid::keyboard_types::Key::Character(s) = &key_event.key {
                    if s == "d" || s == "D" {
                        data.debug = !data.debug;
                        ctx.request_paint();
                    } else if s == "p" || s == "P" {
                        data.paused = !data.paused;
                        ctx.request_paint();
                    } else if s == "q" || s == "Q" {
                        // Submit the QUIT_APP command to exit the application
                        ctx.submit_command(commands::QUIT_APP);
                    } else if s == "w" || s == "W" {
                        data.wireframe = !data.wireframe;
                        ctx.request_paint();
                    }
                }
            }
            Event::MouseDown(mouse_event) => {
                self.last_mouse_pos = mouse_event.pos;
                match mouse_event.button {
                    druid::MouseButton::Left => {
                        self.dragging_rotation = true;
                    }
                    druid::MouseButton::Right => {
                        self.dragging_translation = true;
                    }
                    _ => {}
                }
                ctx.set_active(true); // Capture mouse events
            }
            Event::MouseMove(mouse_event) => {
                if self.dragging_rotation {
                    let delta = mouse_event.pos - self.last_mouse_pos;
                    // Update rotation angles based on mouse movement
                    data.angle_x += delta.y * 0.01; // Adjust sensitivity as needed
                    data.angle_y += delta.x * 0.01;
                    self.last_mouse_pos = mouse_event.pos;
                    ctx.request_paint();
                } else if self.dragging_translation {
                    let delta = mouse_event.pos - self.last_mouse_pos;
                    // Update translation based on mouse movement
                    data.translation[0] += delta.x;
                    data.translation[1] += delta.y;
                    self.last_mouse_pos = mouse_event.pos;
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                match mouse_event.button {
                    druid::MouseButton::Left => {
                        self.dragging_rotation = false;
                    }
                    druid::MouseButton::Right => {
                        self.dragging_translation = false;
                    }
                    _ => {}
                }
                ctx.set_active(false);
            }
            Event::Wheel(wheel_event) => {
                let delta = wheel_event.wheel_delta.y;
                data.zoom *= 1.0 + delta * 0.001;
                data.zoom = data.zoom.clamp(0.1, 10.0); // Clamp zoom level
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
    }
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}
    /// Determines the layout constraints for the cube widget
    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    /// Paint the cube widget
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        // Update FPS calculation
        self.frames_since_last_update += 1;
        let now = Instant::now();
        let duration = now.duration_since(self.last_fps_calculation);
        if duration.as_secs_f64() >= 1.0 {
            self.fps = self.frames_since_last_update as f64 / duration.as_secs_f64();
            self.frames_since_last_update = 0;
            self.last_fps_calculation = now;
        }

        let size = ctx.size();
        let width = size.width as usize;
        let height = size.height as usize;
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let scale = (size.height.min(size.width) / 4.0) * data.zoom; // Adjusted scale

        // Create pixel buffer and z-buffer
        let mut pixel_data = vec![0u8; width * height * 4];
        let mut z_buffer = vec![std::f64::INFINITY; width * height];

        // Define cube vertices
        let vertices = [
            (-1.0, -1.0, -1.0), // 0
            (1.0, -1.0, -1.0),  // 1
            (1.0, 1.0, -1.0),   // 2
            (-1.0, 1.0, -1.0),  // 3
            (-1.0, -1.0, 1.0),  // 4
            (1.0, -1.0, 1.0),   // 5
            (1.0, 1.0, 1.0),    // 6
            (-1.0, 1.0, 1.0),   // 7
        ];

        // Define cube faces (each face is defined by 4 vertex indices)
        let faces = [
            (0, 1, 2, 3),
            (5, 4, 7, 6),
            (4, 0, 3, 7),
            (1, 5, 6, 2),
            (4, 5, 1, 0),
            (3, 2, 6, 7),
        ];

        // Define cube edges (pairs of vertex indices)
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0), // Front face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4), // Back face
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // Connecting edges
        ];

        // Define face colors
        let face_colors = [
            Color::rgb8(255, 0, 0),   // Red
            Color::rgb8(0, 255, 0),   // Green
            Color::rgb8(0, 0, 255),   // Blue
            Color::rgb8(255, 255, 0), // Yellow
            Color::rgb8(255, 0, 255), // Magenta
            Color::rgb8(0, 255, 255), // Cyan
        ];

        // Light source position in world space
        let light_pos_world = [2.0, 2.0, -5.0];

        // Rotation matrices
        let (sin_x, cos_x) = data.angle_x.sin_cos();
        let (sin_y, cos_y) = data.angle_y.sin_cos();

        let rotation_x = [
            [1.0, 0.0, 0.0],
            [0.0, cos_x, -sin_x],
            [0.0, sin_x, cos_x],
        ];

        let rotation_y = [
            [cos_y, 0.0, sin_y],
            [0.0, 1.0, 0.0],
            [-sin_y, 0.0, cos_y],
        ];

        // Combine rotations
        let rotation_matrix = multiply_matrices(&rotation_y, &rotation_x);

        // Transform and project vertices
        let transformed_vertices: Vec<[f64; 3]> = vertices
            .iter()
            .map(|&(x, y, z)| {
                let rotated = multiply_matrix_vector(&rotation_matrix, &[x, y, z]);
                // Apply translation in 3D space
                [
                    rotated[0] + data.translation[0] / scale,
                    rotated[1] + data.translation[1] / scale,
                    rotated[2],
                ]
            })
            .collect();

        // Compute vertex normals
        let mut vertex_normals = vec![[0.0; 3]; vertices.len()];
        for &(a, b, c, d) in faces.iter() {
            let normal = calculate_normal(
                &transformed_vertices[a],
                &transformed_vertices[b],
                &transformed_vertices[c],
            );
            for &index in &[a, b, c, d] {
                vertex_normals[index][0] += normal[0];
                vertex_normals[index][1] += normal[1];
                vertex_normals[index][2] += normal[2];
            }
        }
        for normal in vertex_normals.iter_mut() {
            let length = (normal[0] * normal[0]
                + normal[1] * normal[1]
                + normal[2] * normal[2])
                .sqrt();
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        }

        // Create vertices with normals and screen positions
        let vertices_with_normals: Vec<Vertex> = transformed_vertices
            .iter()
            .zip(vertex_normals.iter())
            .map(|(&position, &normal)| {
                let screen_x = position[0] * scale + center.x;
                let screen_y = position[1] * scale + center.y;
                Vertex {
                    position,
                    screen_position: [screen_x, screen_y],
                    normal,
                }
            })
            .collect();

        if data.wireframe {
            // Draw edges
            for &(start, end) in &edges {
                let v0 = &vertices_with_normals[start];
                let v1 = &vertices_with_normals[end];
                draw_line(
                    v0.screen_position[0],
                    v0.screen_position[1],
                    v1.screen_position[0],
                    v1.screen_position[1],
                    &mut pixel_data,
                    width,
                    height,
                    Color::WHITE,
                );
            }
        } else {
            // Draw faces
            for (face_index, &(a, b, c, d)) in faces.iter().enumerate() {
                // Triangle 1: a, b, c
                draw_triangle(
                    &vertices_with_normals[a],
                    &vertices_with_normals[b],
                    &vertices_with_normals[c],
                    &mut pixel_data,
                    &mut z_buffer,
                    width,
                    height,
                    &light_pos_world,
                    face_colors[face_index],
                );
                // Triangle 2: a, c, d
                draw_triangle(
                    &vertices_with_normals[a],
                    &vertices_with_normals[c],
                    &vertices_with_normals[d],
                    &mut pixel_data,
                    &mut z_buffer,
                    width,
                    height,
                    &light_pos_world,
                    face_colors[face_index],
                );
            }
        }

        // Create and draw the image
        let image = ctx
            .make_image(width, height, &pixel_data, druid::piet::ImageFormat::RgbaSeparate)
            .unwrap();
        ctx.draw_image(&image, size.to_rect(), InterpolationMode::NearestNeighbor);

        // Add debug info if debug mode is enabled
        if data.debug {
            // Draw program name and version
            let text = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 10.0));

            // Draw angles
            let text = format!(
                "Angle X: {:.2}, Angle Y: {:.2}",
                data.angle_x, data.angle_y
            );
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 30.0));

            // Draw translation
            let text = format!(
                "Translation X: {:.2}, Y: {:.2}",
                data.translation[0], data.translation[1]
            );
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 50.0));

            // Draw light position
            let text = format!(
                "Light: ({:.2}, {:.2}, {:.2})",
                light_pos_world[0], light_pos_world[1], light_pos_world[2]
            );
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 70.0));

            // Draw FPS
            let text = format!("FPS: {:.2}", self.fps);
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 90.0));

            // Draw zoom level
            let text = format!("Zoom: {:.2}", data.zoom);
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 110.0));
        }

        // Display 'Paused' if the simulation is paused
        if data.paused {
            // Draw a semi-transparent overlay
            let overlay_color = Color::rgba8(0, 0, 0, 150); // Adjust the alpha value as needed
            ctx.fill(size.to_rect(), &overlay_color);

            // Draw 'Paused' text
            let text = "Paused";
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 36.0)
                .default_attribute(druid::piet::FontWeight::BOLD)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            let text_size = text_layout.size();
            let pos = (
                (size.width - text_size.width) / 2.0,
                (size.height - text_size.height) / 2.0,
            );
            ctx.draw_text(&text_layout, pos);
        }
    }
}

/// Vertex structure with position, screen position, and normal
struct Vertex {
    position: [f64; 3],
    screen_position: [f64; 2],
    normal: [f64; 3],
}

/// Draws a triangle with per-pixel lighting
fn draw_triangle(
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    pixel_data: &mut [u8],
    z_buffer: &mut [f64],
    width: usize,
    height: usize,
    light_pos_world: &[f64; 3],
    base_color: Color,
) {
    // Compute bounding box of the triangle
    let min_x = v0
        .screen_position[0]
        .min(v1.screen_position[0])
        .min(v2.screen_position[0])
        .floor()
        .max(0.0) as usize;
    let max_x = v0
        .screen_position[0]
        .max(v1.screen_position[0])
        .max(v2.screen_position[0])
        .ceil()
        .min(width as f64 - 1.0) as usize;
    let min_y = v0
        .screen_position[1]
        .min(v1.screen_position[1])
        .min(v2.screen_position[1])
        .floor()
        .max(0.0) as usize;
    let max_y = v0
        .screen_position[1]
        .max(v1.screen_position[1])
        .max(v2.screen_position[1])
        .ceil()
        .min(height as f64 - 1.0) as usize;

    // Precompute area of the triangle
    let area = edge_function(&v0.screen_position, &v1.screen_position, &v2.screen_position);

    // For each pixel in the bounding box
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            let p = [px, py];

            let w0 = edge_function(&v1.screen_position, &v2.screen_position, &p);
            let w1 = edge_function(&v2.screen_position, &v0.screen_position, &p);
            let w2 = edge_function(&v0.screen_position, &v1.screen_position, &p);

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                // Inside triangle
                // Normalize barycentric coordinates
                let w0 = w0 / area;
                let w1 = w1 / area;
                let w2 = w2 / area;

                // Interpolate position
                let px3d = v0.position[0] * w0 + v1.position[0] * w1 + v2.position[0] * w2;
                let py3d = v0.position[1] * w0 + v1.position[1] * w1 + v2.position[1] * w2;
                let pz3d = v0.position[2] * w0 + v1.position[2] * w1 + v2.position[2] * w2;

                // Depth test
                let offset = y * width + x;
                if pz3d < z_buffer[offset] {
                    z_buffer[offset] = pz3d;

                    // Interpolate normal
                    let nx = v0.normal[0] * w0 + v1.normal[0] * w1 + v2.normal[0] * w2;
                    let ny = v0.normal[1] * w0 + v1.normal[1] * w1 + v2.normal[1] * w2;
                    let nz = v0.normal[2] * w0 + v1.normal[2] * w1 + v2.normal[2] * w2;
                    let length = (nx * nx + ny * ny + nz * nz).sqrt();
                    let interpolated_normal = [nx / length, ny / length, nz / length];

                    // Compute lighting
                    let light_intensity = calculate_light_intensity(
                        &interpolated_normal,
                        &[px3d, py3d, pz3d],
                        light_pos_world,
                    );

                    // Compute shaded color
                    let shaded_color = apply_lighting(base_color.clone(), light_intensity);

                    // Set pixel color
                    let pixel_offset = offset * 4;
                    let (r, g, b, a) = shaded_color.as_rgba8();
                    pixel_data[pixel_offset] = r;
                    pixel_data[pixel_offset + 1] = g;
                    pixel_data[pixel_offset + 2] = b;
                    pixel_data[pixel_offset + 3] = a;
                }
            }
        }
    }
}

/// Edge function used in rasterization
fn edge_function(a: &[f64; 2], b: &[f64; 2], c: &[f64; 2]) -> f64 {
    (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])
}

/// Multiplies a 3x3 matrix by a 3-dimensional vector
fn multiply_matrix_vector(matrix: &[[f64; 3]; 3], vector: &[f64; 3]) -> [f64; 3] {
    let mut result = [0.0; 3];
    for i in 0..3 {
        for j in 0..3 {
            result[i] += matrix[i][j] * vector[j];
        }
    }
    result
}

/// Multiplies two 3x3 matrices
fn multiply_matrices(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut result = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

/// Calculates the normal vector of a triangle
fn calculate_normal(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> [f64; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    [normal[0] / length, normal[1] / length, normal[2] / length]
}

/// Calculates the light intensity based on the normal vector and light position
fn calculate_light_intensity(
    normal: &[f64; 3],
    position: &[f64; 3],
    light_pos: &[f64; 3],
) -> f64 {
    let light_dir = [
        light_pos[0] - position[0],
        light_pos[1] - position[1],
        light_pos[2] - position[2],
    ];
    let length = (light_dir[0] * light_dir[0]
        + light_dir[1] * light_dir[1]
        + light_dir[2] * light_dir[2])
        .sqrt();
    let light_dir = [
        light_dir[0] / length,
        light_dir[1] / length,
        light_dir[2] / length,
    ];
    let dot_product =
        normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2];
    dot_product.max(0.1) // Ensure a minimum ambient light
}

/// Applies lighting to a color
fn apply_lighting(color: Color, intensity: f64) -> Color {
    let r = (color.as_rgba8().0 as f64 * intensity).min(255.0) as u8;
    let g = (color.as_rgba8().1 as f64 * intensity).min(255.0) as u8;
    let b = (color.as_rgba8().2 as f64 * intensity).min(255.0) as u8;
    Color::rgb8(r, g, b)
}

/// Draws a line between two points in the pixel buffer using Bresenham's algorithm
fn draw_line(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    pixel_data: &mut [u8],
    width: usize,
    height: usize,
    color: Color,
) {
    let (mut x0, mut y0, x1, y1) = (
        x0.round() as isize,
        y0.round() as isize,
        x1.round() as isize,
        y1.round() as isize,
    );
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy; // error value e_xy

    loop {
        if x0 >= 0 && x0 < width as isize && y0 >= 0 && y0 < height as isize {
            let offset = (y0 as usize * width + x0 as usize) * 4;
            let (r, g, b, a) = color.as_rgba8();
            pixel_data[offset] = r;
            pixel_data[offset + 1] = g;
            pixel_data[offset + 2] = b;
            pixel_data[offset + 3] = a;
        }

        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Main function
pub fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(CubeWidget::new())
        .title(LocalizedString::new("3D Cube with Per-Pixel Lighting"))
        .window_size((400.0, 400.0));

    let initial_state = AppState {
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
