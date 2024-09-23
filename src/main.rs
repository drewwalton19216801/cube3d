//! # 3D Cube Renderer
//! 
//! This program renders a 3D cube using the Druid GUI framework. The cube rotates
//! continuously and features shaded faces with basic lighting effects. It demonstrates
//! 3D graphics concepts such as projection, rotation, and simple lighting calculations.
//! 
//! ## Features:
//! - Rotating 3D cube visualization
//! - Shaded faces with basic lighting
//! - Debug mode for displaying additional information
//! - Command-line argument parsing for enabling debug mode

use clap::Parser;
use druid::kurbo::{Point, BezPath};
use druid::text::FontFamily;
use druid::widget::prelude::*;
use druid::{AppLauncher, Color, Data, LocalizedString, PlatformError, RenderContext, Widget, WindowDesc};
use std::f64::consts::PI;
use druid::piet::{Text, TextLayoutBuilder};

/// Application state
#[derive(Clone, Data)]
struct AppState {
    /// Current rotation angle of the cube
    angle: f64,
    /// Enable debug mode
    debug: bool,
}

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

/// 3D cube widget
struct CubeWidget;

impl Widget<AppState> for CubeWidget {
    /// Handle events for the cube widget
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_timer(std::time::Duration::from_millis(16));
            }
            Event::Timer(_) => {
                data.angle += 0.02;
                if data.angle > 2.0 * PI {
                    data.angle -= 2.0 * PI;
                }
                ctx.request_timer(std::time::Duration::from_millis(16));
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}

    /// Determines the layout constraints for the cube widget
    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        bc.max()
    }

    /// Paint the cube widget
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
        let size = ctx.size();
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let scale = size.height.min(size.width) / 4.0;

        // Define cube vertices
        let vertices = [
            (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0), (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
            (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0), (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0),
        ];

        // Define cube faces (each face is defined by 4 vertex indices)
        let faces = [
            (0, 1, 2, 3), (5, 4, 7, 6), // front and back
            (4, 0, 3, 7), (1, 5, 6, 2), // left and right
            (4, 5, 1, 0), (3, 2, 6, 7), // top and bottom
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

        // Light source position (fixed in 3D space) is in the center of the cube, slightly offset to the left
        let light_pos = [center.x - 0.5, center.y, center.x];

        // Rotation matrices
        let (sin_a, cos_a) = data.angle.sin_cos();
        let rotation_y = [
            [cos_a, 0.0, sin_a],
            [0.0, 1.0, 0.0],
            [-sin_a, 0.0, cos_a],
        ];
        let rotation_x = [
            [1.0, 0.0, 0.0],
            [0.0, cos_a, -sin_a],
            [0.0, sin_a, cos_a],
        ];

        // Project and transform 3D points to 2D
        let transformed_vertices: Vec<[f64; 3]> = vertices.iter().map(|&(x, y, z)| {
            let [x, y, z] = multiply_matrix_vector(&rotation_y, &[x, y, z]);
            multiply_matrix_vector(&rotation_x, &[x, y, z])
        }).collect();

        let projected_vertices: Vec<Point> = transformed_vertices.iter().map(|&[x, y, _z]| {
            Point::new(x * scale + center.x, y * scale + center.y)
        }).collect();

        // Calculate face normals and sort faces by z-depth
        let mut face_data: Vec<(usize, [f64; 3], f64)> = faces.iter().enumerate().map(|(i, &(a, b, c, d))| {
            let normal = calculate_normal(&transformed_vertices[a], &transformed_vertices[b], &transformed_vertices[c]);
            let avg_z = (transformed_vertices[a][2] + transformed_vertices[b][2] + transformed_vertices[c][2] + transformed_vertices[d][2]) / 4.0;
            (i, normal, avg_z)
        }).collect();
        face_data.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Draw faces
        for (face_index, normal, _) in face_data {
            let (a, b, c, d) = faces[face_index];
            let mut path = BezPath::new();
            path.move_to(projected_vertices[a]);
            path.line_to(projected_vertices[b]);
            path.line_to(projected_vertices[c]);
            path.line_to(projected_vertices[d]);
            path.close_path();

            // Calculate lighting
            let light_intensity = calculate_light_intensity(&normal, &light_pos);
            let base_color = face_colors[face_index];
            let shaded_color = apply_lighting(base_color, light_intensity);

            ctx.fill(path, &shaded_color);
        }

        // Add debug info to the top left if debug mode is enabled
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

            // Draw angle
            let text = format!("Angle: {:.2}", data.angle);
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 30.0));

            // Draw light position
            let text = format!("Light: ({:.2}, {:.2}, {:.2})", light_pos[0], light_pos[1], light_pos[2]);
            let text_layout = ctx
                .text()
                .new_text_layout(text)
                .font(FontFamily::SYSTEM_UI, 12.0)
                .text_color(Color::WHITE)
                .build()
                .unwrap();
            ctx.draw_text(&text_layout, (10.0, 50.0));
        }
    }
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
fn calculate_light_intensity(normal: &[f64; 3], light_pos: &[f64; 3]) -> f64 {
    let light_dir = normalize(light_pos);
    let dot_product = normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2];
    dot_product.max(0.1) // Ensure a minimum ambient light
}

/// Normalizes a 3-dimensional vector
fn normalize(v: &[f64; 3]) -> [f64; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / length, v[1] / length, v[2] / length]
}

/// Applies lighting to a color
fn apply_lighting(color: Color, intensity: f64) -> Color {
    let r = (color.as_rgba8().0 as f64 * intensity).min(255.0) as u8;
    let g = (color.as_rgba8().1 as f64 * intensity).min(255.0) as u8;
    let b = (color.as_rgba8().2 as f64 * intensity).min(255.0) as u8;
    Color::rgb8(r, g, b)
}

/// Main function
pub fn main() -> Result<(), PlatformError> {
    let args = Args::parse();

    let main_window = WindowDesc::new(CubeWidget)
        .title(LocalizedString::new("3D Cube with Improved Shaded Faces"))
        .window_size((400.0, 400.0));

    let initial_state = AppState { angle: 0.0, debug: args.debug };

    AppLauncher::with_window(main_window)
        .launch(initial_state)?;

    Ok(())
}