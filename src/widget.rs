use crate::graphics::{draw_line, draw_triangle};
use crate::math::{calculate_normal, multiply_matrices, multiply_matrix_vector, point_in_triangle};
use crate::state::AppState;
use crate::vertex::Vertex;
use druid::kurbo::Point;
use druid::text::FontFamily;
use druid::widget::prelude::*;
use druid::{
    commands,
    piet::{InterpolationMode, Text, TextLayout, TextLayoutBuilder},
    Color, RenderContext, Widget,
};
use std::time::Instant;

/// 3D cube widget
pub struct CubeWidget {
    frames_since_last_update: usize,
    last_fps_calculation: Instant,
    fps: f64,
    /// Is the user currently dragging for rotation?
    dragging_rotation: bool,
    /// Is the user currently dragging for translation?
    dragging_translation: bool,
    /// Last mouse position
    last_mouse_pos: Point,
    /// Widget size
    size: Size,
}

impl CubeWidget {
    pub fn new() -> Self {
        CubeWidget {
            frames_since_last_update: 0,
            last_fps_calculation: Instant::now(),
            fps: 0.0,
            dragging_rotation: false,
            dragging_translation: false,
            last_mouse_pos: Point::ZERO,
            size: Size::ZERO,
        }
    }

    /// Computes the projected vertices for the current state
    fn compute_projected_vertices(&self, data: &AppState) -> Vec<Vertex> {
        let center = Point::new(self.size.width / 2.0, self.size.height / 2.0);
        let scale = (self.size.height.min(self.size.width) / 4.0) * data.zoom; // Adjusted scale

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

        // Rotation matrices
        let (sin_x, cos_x) = data.angle_x.sin_cos();
        let (sin_y, cos_y) = data.angle_y.sin_cos();

        let rotation_x = [[1.0, 0.0, 0.0], [0.0, cos_x, -sin_x], [0.0, sin_x, cos_x]];

        let rotation_y = [[cos_y, 0.0, sin_y], [0.0, 1.0, 0.0], [-sin_y, 0.0, cos_y]];

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
        let faces = [
            (0, 1, 2, 3),
            (5, 4, 7, 6),
            (4, 0, 3, 7),
            (1, 5, 6, 2),
            (4, 5, 1, 0),
            (3, 2, 6, 7),
        ];

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
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
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

        vertices_with_normals
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
                    match s.as_str() {
                        "d" | "D" => {
                            data.debug = !data.debug;
                            ctx.request_paint();
                        }
                        "p" | "P" => {
                            data.paused = !data.paused;
                            // Reset any mouse events that were captured
                            self.last_mouse_pos = Point::ZERO;
                            self.dragging_rotation = false;
                            self.dragging_translation = false;
                            ctx.request_paint();
                        }
                        "q" | "Q" => {
                            // Submit the QUIT_APP command to exit the application
                            ctx.submit_command(commands::QUIT_APP);
                        }
                        "w" | "W" => {
                            if !data.paused {
                                data.wireframe = !data.wireframe;
                                ctx.request_paint();
                            }
                        }
                        "r" | "R" => {
                            if !data.paused {
                                // Reset to default values
                                data.angle_x = 0.0;
                                data.angle_y = 0.0;
                                data.translation = [0.0, 0.0];
                                data.zoom = 1.0;
                                data.wireframe = false;
                                ctx.request_paint();
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::MouseDown(mouse_event) => {
                if !data.paused {
                    self.last_mouse_pos = mouse_event.pos;
                    // Compute projected vertices
                    let vertices_with_normals = self.compute_projected_vertices(data);

                    // Define cube faces (each face is defined by 4 vertex indices)
                    let faces = [
                        (0, 1, 2, 3),
                        (5, 4, 7, 6),
                        (4, 0, 3, 7),
                        (1, 5, 6, 2),
                        (4, 5, 1, 0),
                        (3, 2, 6, 7),
                    ];

                    let mut clicked_inside_cube = false;
                    let click_point = [mouse_event.pos.x, mouse_event.pos.y];

                    for &(a, b, c, d) in &faces {
                        // Triangle 1: a, b, c
                        let v0 = &vertices_with_normals[a];
                        let v1 = &vertices_with_normals[b];
                        let v2 = &vertices_with_normals[c];
                        if point_in_triangle(
                            click_point,
                            v0.screen_position,
                            v1.screen_position,
                            v2.screen_position,
                        ) {
                            clicked_inside_cube = true;
                            break;
                        }
                        // Triangle 2: a, c, d
                        let v0 = &vertices_with_normals[a];
                        let v1 = &vertices_with_normals[c];
                        let v2 = &vertices_with_normals[d];
                        if point_in_triangle(
                            click_point,
                            v0.screen_position,
                            v1.screen_position,
                            v2.screen_position,
                        ) {
                            clicked_inside_cube = true;
                            break;
                        }
                    }

                    if clicked_inside_cube {
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
                }
            }
            Event::MouseMove(mouse_event) => {
                if !data.paused {
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
            }
            Event::MouseUp(mouse_event) => {
                if !data.paused {
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
            }
            Event::Wheel(wheel_event) => {
                if !data.paused {
                    let delta = wheel_event.wheel_delta.y;
                    data.zoom *= 1.0 + delta * 0.001;
                    data.zoom = data.zoom.clamp(0.1, 10.0); // Clamp zoom level
                    ctx.request_paint();
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
        if let LifeCycle::Size(size) = event {
            self.size = *size;
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {
    }

    /// Determines the layout constraints for the cube widget
    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        let size = bc.max();
        self.size = size;
        size
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

        // Create pixel buffer and z-buffer
        let mut pixel_data = vec![0u8; width * height * 4];
        let mut z_buffer = vec![std::f64::INFINITY; width * height];

        // Compute projected vertices
        let vertices_with_normals = self.compute_projected_vertices(data);

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
        let light_pos_world = data.light_position;

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
            .make_image(
                width,
                height,
                &pixel_data,
                druid::piet::ImageFormat::RgbaSeparate,
            )
            .unwrap();
        ctx.draw_image(&image, size.to_rect(), InterpolationMode::NearestNeighbor);

        // Add debug info if debug mode is enabled
        if data.debug {
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
            let text = format!("Angle X: {:.2}, Angle Y: {:.2}", data.angle_x, data.angle_y);
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
