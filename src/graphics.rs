use crate::math::{apply_lighting, calculate_light_intensity, edge_function, normalize};
use crate::vertex::Vertex;
use druid::Color;

/// Checks if a triangle should be back-face culled
/// Returns true if the triangle faces away from the camera
#[inline(always)]
pub fn should_backface_cull(v0: &Vertex, v1: &Vertex, v2: &Vertex) -> bool {
    // Calculate the normal of the triangle in screen space
    let dx1 = v1.screen_position[0] - v0.screen_position[0];
    let dy1 = v1.screen_position[1] - v0.screen_position[1];
    let dx2 = v2.screen_position[0] - v0.screen_position[0];
    let dy2 = v2.screen_position[1] - v0.screen_position[1];
    
    // Cross product gives us the normal (pointing out of the screen)
    // In screen space, Y increases downward, so we need to adjust the sign
    // If this is positive, the triangle faces away from the camera
    (dx1 * dy2 - dx2 * dy1) > 0.0
}

/// Checks if a triangle is completely outside the viewport
/// Returns true if the triangle should be culled
#[inline(always)]
pub fn should_viewport_cull(v0: &Vertex, v1: &Vertex, v2: &Vertex, width: usize, height: usize) -> bool {
    let width_f = width as f32;
    let height_f = height as f32;
    
    // Check if all vertices are outside the same edge of the viewport
    let all_left = v0.screen_position[0] < 0.0 && v1.screen_position[0] < 0.0 && v2.screen_position[0] < 0.0;
    let all_right = v0.screen_position[0] >= width_f && v1.screen_position[0] >= width_f && v2.screen_position[0] >= width_f;
    let all_top = v0.screen_position[1] < 0.0 && v1.screen_position[1] < 0.0 && v2.screen_position[1] < 0.0;
    let all_bottom = v0.screen_position[1] >= height_f && v1.screen_position[1] >= height_f && v2.screen_position[1] >= height_f;
    
    all_left || all_right || all_top || all_bottom
}

/// Checks if a triangle is behind the camera (negative Z)
/// Returns true if the triangle should be culled
#[inline(always)]
pub fn should_depth_cull(v0: &Vertex, v1: &Vertex, v2: &Vertex) -> bool {
    // If all vertices are behind the camera (negative Z), cull the triangle
    v0.position[2] < 0.0 && v1.position[2] < 0.0 && v2.position[2] < 0.0
}

/// Performs all culling tests on a triangle
/// Returns true if the triangle should be culled
#[inline(always)]
pub fn should_cull_triangle(v0: &Vertex, v1: &Vertex, v2: &Vertex, width: usize, height: usize) -> bool {
    should_depth_cull(v0, v1, v2) ||
    should_viewport_cull(v0, v1, v2, width, height) ||
    should_backface_cull(v0, v1, v2)
}

/// Draws a triangle with per-pixel lighting
pub fn draw_triangle(
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    pixel_data: &mut [u8],
    z_buffer: &mut [f32],
    width: usize,
    height: usize,
    light_pos_world: &[f32; 3],
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
        .min(width as f32 - 1.0) as usize;
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
        .min(height as f32 - 1.0) as usize;

    // Precompute area of the triangle
    let area = edge_function(&v0.screen_position, &v1.screen_position, &v2.screen_position);

    // For each pixel in the bounding box
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let p = [px, py];

            // Calculate edge functions ONCE
            let w0 = edge_function(&v1.screen_position, &v2.screen_position, &p);
            let w1 = edge_function(&v2.screen_position, &v0.screen_position, &p);
            let w2 = edge_function(&v0.screen_position, &v1.screen_position, &p);

            // Check if inside triangle using pre-computed values
            let inside = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) ||
                         (w0 < 0.0 && w1 < 0.0 && w2 < 0.0);

            if inside {
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
                    let interpolated_normal = normalize(&[nx, ny, nz]);

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

/// Draws a line between two points in the pixel buffer using Bresenham's algorithm
pub fn draw_line(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
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
