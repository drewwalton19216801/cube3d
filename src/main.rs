//! # 3D Cube Renderer
//! 
//! This program renders a rotating 3D cube in the terminal using ASCII characters.
//! It employs basic 3D graphics techniques such as rotation, projection, and simple lighting.
//!
//! ## Features
//! - Renders a 3D cube that rotates continuously
//! - Adjusts to terminal window size changes
//! - Implements basic lighting for a more realistic appearance
//! - Falls back to wireframe rendering during rapid resizing for performance
//!
//! ## Dependencies
//! - `crossterm` for terminal manipulation and drawing
//! - Standard Rust libraries for timing and mathematical operations

use std::io::{stdout, Write, Result};
use std::time::{Duration, Instant};
use std::{thread, f32::consts::PI};
use crossterm::style::Print;
use crossterm::{
    execute,
    style::{Color, SetForegroundColor, ResetColor},
    terminal::{Clear, ClearType, size, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show, MoveTo},
    event::{poll, read, Event, KeyCode},
};

/// Constants for cube rendering and animation
const DISTANCE: f32 = 50.0;
const ANGLE_INCREMENT: f32 = 0.05;
const MIN_CUBE_SIZE: f32 = 4.0;
const LIGHT_DIRECTION: Point3D = Point3D { x: -1.0, y: -1.0, z: -1.0 };
const FRAME_DURATION: Duration = Duration::from_millis(33); // ~30 FPS

/// Represents a point in 3D space
#[derive(Clone, Copy)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

/// Represents a point in 2D space (projected on the terminal)
#[derive(Clone, Copy)]
struct Point2D {
    x: i32,
    y: i32,
}

/// Represents a face of the cube
struct Face {
    vertices: [usize; 4],
    normal: Point3D,
    color: Color,
}

struct Buffer {
    width: usize,
    height: usize,
    content: Vec<Vec<(char, Color)>>,
}

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            content: vec![vec![(' ', Color::Reset); width]; height],
        }
    }

    fn set(&mut self, x: usize, y: usize, ch: char, color: Color) {
        if x < self.width && y < self.height {
            self.content[y][x] = (ch, color);
        }
    }

    fn clear(&mut self) {
        for row in self.content.iter_mut() {
            for cell in row.iter_mut() {
                *cell = (' ', Color::Reset);
            }
        }
    }

    fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.content = vec![vec![(' ', Color::Reset); new_width]; new_height];
    }

    fn render(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        let mut last_color = Color::Reset;
        for (y, row) in self.content.iter().enumerate() {
            execute!(stdout, MoveTo(0, y as u16))?;
            for &(ch, color) in row {
                if color != last_color {
                    execute!(stdout, SetForegroundColor(color))?;
                    last_color = color;
                }
                write!(stdout, "{}", ch)?;
            }
        }
        execute!(stdout, ResetColor)?;
        stdout.flush()?;
        Ok(())
    }
}

/// Main function that sets up and runs the cube rendering loop
///
/// This function is the entry point of the program. It initializes the terminal, creates the cube and faces,
/// and starts the main loop that updates and renders the cube.
///
/// The main loop of the program is an infinite loop that sleeps for a short duration when not rendering a frame,
/// and renders a frame if enough time has passed and the terminal hasn't been resized recently.
///
/// If the terminal has been resized recently, the loop renders the cube in wireframe mode.
///
/// Otherwise, it renders the cube with shading.
fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide, Clear(ClearType::All))?;

    let mut angle_x = 0.0;
    let mut angle_y = 0.0;
    let mut last_frame = Instant::now();
    let light_direction = normalize(&LIGHT_DIRECTION);

    let (mut width, mut height) = size()?;
    let mut buffer = Buffer::new(width as usize, height as usize);

    let mut is_resizing = false;
    let mut last_resize_time = Instant::now();
    let resize_cooldown = Duration::from_millis(500); // Increased cooldown period

    // Draw the welcome screen
    draw_welcome_message(&mut buffer, width, height);
    buffer.render(&mut stdout)?;

    // Wait 3 seconds before starting the cube animation
    std::thread::sleep(Duration::from_secs(3));
    buffer.clear();
    buffer.render(&mut stdout)?;

    loop {
        if poll(Duration::from_millis(1))? {
            match read()? {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') {
                        break;
                    }
                },
                Event::Resize(new_width, new_height) => {
                    width = new_width;
                    height = new_height;
                    buffer.resize(width as usize, height as usize);
                    execute!(stdout, Clear(ClearType::All))?;
                    is_resizing = true;
                    last_resize_time = Instant::now();
                },
                _ => {}
            }
        }

        let now = Instant::now();

        if is_resizing {
            if now.duration_since(last_resize_time) > resize_cooldown {
                is_resizing = false;
            } else {
                buffer.clear();
                draw_resize_message(&mut buffer, width, height);
                buffer.render(&mut stdout)?;
                continue;
            }
        }

        let elapsed = now.duration_since(last_frame);

        if elapsed >= FRAME_DURATION {
            if width < 10 || height < 10 {
                execute!(stdout, Clear(ClearType::All), MoveTo(0, 0), Print("Terminal too small"))?;
                stdout.flush()?;
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            let center_x = width as i32 / 2;
            let center_y = height as i32 / 2;
            let cube_size = (width.min(height) as f32 * 0.4).max(MIN_CUBE_SIZE);

            let cube = create_cube(cube_size);
            let faces = create_faces();
            let rotated_cube = rotate_cube(&cube, angle_x, angle_y);
            let projected_cube = project_cube(&rotated_cube, center_x, center_y);

            buffer.clear();
            draw_cube(&mut buffer, &projected_cube, &rotated_cube, &faces, &light_direction, angle_x, angle_y, width, height)?;
            buffer.render(&mut stdout)?;

            angle_x += ANGLE_INCREMENT;
            angle_y += ANGLE_INCREMENT * 0.7;
            if angle_x >= 2.0 * PI {
                angle_x -= 2.0 * PI;
            }
            if angle_y >= 2.0 * PI {
                angle_y -= 2.0 * PI;
            }

            last_frame = now;
        }

        thread::sleep(Duration::from_millis(1));
    }

    execute!(stdout, Show)?;
    disable_raw_mode()?;
    Ok(())
}


/// Draws a welcome screen when the program starts
/// The welcome screen displays a message indicating that the program is running
/// and tells the user to press 'q' to quit
fn draw_welcome_message(buffer: &mut Buffer, width: u16, height: u16) {
    let welcome_message = "Welcome to cube3d, press 'q' to quit";
    let x = width as usize / 2 - welcome_message.len() / 2;
    let y = height as usize / 2;

    for (i, ch) in welcome_message.chars().enumerate() {
        buffer.set(x + i, y, ch, Color::White);
    }
}

/// Draws a message indicating that the terminal is resizing
fn draw_resize_message(buffer: &mut Buffer, width: u16, height: u16) {
    let message = "Resizing...";
    let x = width as usize / 2 - message.len() / 2;
    let y = height as usize / 2;

    for (i, ch) in message.chars().enumerate() {
        buffer.set(x + i, y, ch, Color::White);
    }
}

/// Creates the initial cube vertices
fn create_cube(size: f32) -> Vec<Point3D> {
    vec![
        Point3D { x: -size/2.0, y: -size/2.0, z: -size/2.0 },
        Point3D { x:  size/2.0, y: -size/2.0, z: -size/2.0 },
        Point3D { x:  size/2.0, y:  size/2.0, z: -size/2.0 },
        Point3D { x: -size/2.0, y:  size/2.0, z: -size/2.0 },
        Point3D { x: -size/2.0, y: -size/2.0, z:  size/2.0 },
        Point3D { x:  size/2.0, y: -size/2.0, z:  size/2.0 },
        Point3D { x:  size/2.0, y:  size/2.0, z:  size/2.0 },
        Point3D { x: -size/2.0, y:  size/2.0, z:  size/2.0 },
    ]
}

/// Creates the faces of the cube
fn create_faces() -> Vec<Face> {
    vec![
        Face { vertices: [0, 1, 2, 3], normal: Point3D { x: 0.0, y: 0.0, z: -1.0 }, color: Color::Red },
        Face { vertices: [5, 4, 7, 6], normal: Point3D { x: 0.0, y: 0.0, z: 1.0 }, color: Color::Green },
        Face { vertices: [1, 5, 6, 2], normal: Point3D { x: 1.0, y: 0.0, z: 0.0 }, color: Color::Blue },
        Face { vertices: [4, 0, 3, 7], normal: Point3D { x: -1.0, y: 0.0, z: 0.0 }, color: Color::Yellow },
        Face { vertices: [3, 2, 6, 7], normal: Point3D { x: 0.0, y: 1.0, z: 0.0 }, color: Color::Magenta },
        Face { vertices: [1, 0, 4, 5], normal: Point3D { x: 0.0, y: -1.0, z: 0.0 }, color: Color::Cyan },
    ]
}

/// Creates the faces of the cube
///
/// This function takes a 3D point and rotates it around the X and Y axes by
/// the given angles. The resulting points are then used to create the faces of
/// the cube.
///
/// # Formula
///
/// The formula for rotating a 3D point around the X axis is:
///
/// y' = y * cos(x) - z * sin(x)
/// z' = y * sin(x) + z * cos(x)
///
/// And the formula for rotating a 3D point around the Y axis is:
///
/// x' = x * cos(y) + z * sin(y)
/// z' = -x * sin(y) + z * cos(y)
///
/// # Parameters
///
/// * `cube`: The 3D points to rotate
/// * `angle_x`: The angle of rotation around the X axis (in radians)
/// * `angle_y`: The angle of rotation around the Y axis (in radians)
///
/// # Returns
///
/// A vector of 3D points, where each point is the rotated version of the
/// corresponding point in the input vector.
fn rotate_cube(cube: &[Point3D], angle_x: f32, angle_y: f32) -> Vec<Point3D> {
    cube.iter()
        .map(|p| {
            // Rotate around X-axis
            let y1 = p.y * angle_x.cos() - p.z * angle_x.sin();
            let z1 = p.y * angle_x.sin() + p.z * angle_x.cos();

            // Rotate around Y-axis
            let x2 = p.x * angle_y.cos() + z1 * angle_y.sin();
            let z2 = -p.x * angle_y.sin() + z1 * angle_y.cos();

            Point3D { x: x2, y: y1, z: z2 }
        })
        .collect()
}

/// Projects 3D points onto 2D space, taking into account the distance and center of the projection.
///
/// # Formula
///
/// The formula is derived from the perspective projection equation. We want to map a point at
/// coordinate (x, y, z) in 3D space to a point at coordinate (x', y') in 2D space. The
/// perspective projection equation is:
///
/// x' = x * d / (z + d)
///
/// y' = y * d / (z + d)
///
/// where d is the distance from the camera to the projection plane.
///
/// # Parameters
///
/// * `cube`: The 3D points to project
/// * `center_x`: The x-coordinate of the center of the projection
/// * `center_y`: The y-coordinate of the center of the projection
///
/// # Returns
///
/// A vector of 2D points, where each point is the projection of the corresponding 3D point in the input vector.
fn project_cube(cube: &[Point3D], center_x: i32, center_y: i32) -> Vec<Point2D> {
    cube.iter()
        .map(|p| {
            let x = (p.x * DISTANCE / (p.z + DISTANCE)) as i32 + center_x;
            let y = (p.y * DISTANCE / (p.z + DISTANCE)) as i32 + center_y;
            Point2D { x, y }
        })
        .collect()
}

/// Draws the cube with shading
///
/// This function first calculates the depth of each face in the cube by
/// taking the average Z-coordinate of its vertices. It then sorts the faces
/// by their depth, so that the faces in the background are drawn first.
///
/// For each face, it calculates the dot product of its normal vector with the
/// light direction. This gives the angle between the face and the light,
/// which is used to determine the shade of the face. The shade is then used
/// to determine the character and color to use when drawing the face.
///
/// The `fill_face` function is used to draw each face. It takes the `stdout`
/// handle, the projected vertices of the cube, the vertices of the face, the
/// shade character, the shade color, and the width and height of the
/// terminal.
fn draw_cube(buffer: &mut Buffer, projected: &[Point2D], rotated: &[Point3D], faces: &[Face], light_direction: &Point3D, angle_x: f32, angle_y: f32, width: u16, height: u16) -> Result<()> {
    let mut face_depths: Vec<(usize, f32)> = faces.iter().enumerate()
        .map(|(i, face)| {
            let center = face_center(rotated, &face.vertices);
            (i, center.z)
        })
        .collect();

    face_depths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (face_index, _) in face_depths {
        let face = &faces[face_index];
        let rotated_normal = rotate_point(&face.normal, angle_x, angle_y);
        let shade = dot_product(&rotated_normal, light_direction).max(0.1);

        let shade_char = get_shade_char(shade);
        let color = shade_color(&face.color, shade);

        fill_face(buffer, projected, &face.vertices, shade_char, color, width, height);
    }

    Ok(())
}

/// Rotates a 3D point around the X and Y axes by the given angles.
///
/// # Formula
///
/// The formula for rotating a 3D point around the X axis is:
///
/// y' = y * cos(x) - z * sin(x)
/// z' = y * sin(x) + z * cos(x)
///
/// And the formula for rotating a 3D point around the Y axis is:
///
/// x' = x * cos(y) + z' * sin(y)
/// z' = -x * sin(y) + z' * cos(y)
///
/// # Parameters
///
/// * `point`: The 3D point to rotate
/// * `angle_x`: The angle of rotation around the X axis (in radians)
/// * `angle_y`: The angle of rotation around the Y axis (in radians)
///
/// # Returns
///
/// The rotated 3D point
fn rotate_point(point: &Point3D, angle_x: f32, angle_y: f32) -> Point3D {
    let cos_x = angle_x.cos();
    let sin_x = angle_x.sin();
    let cos_y = angle_y.cos();
    let sin_y = angle_y.sin();

    // Rotate around X axis
    let y1 = point.y * cos_x - point.z * sin_x;
    let z1 = point.y * sin_x + point.z * cos_x;

    // Rotate around Y axis
    let x2 = point.x * cos_y + z1 * sin_y;
    let z2 = -point.x * sin_y + z1 * cos_y;

    Point3D { x: x2, y: y1, z: z2 }
}

/// Fills a face with a shade character
///
/// This function takes a list of projected vertices for a face, and fills
/// the face with a shade character. The shade character is chosen based on
/// the shade value of the face, and its color is determined by the color
/// of the face.
///
/// The function draws the face by finding the intersection points of the
/// edges of the face with the horizontal lines of the terminal, and then
/// drawing a line between each pair of consecutive intersection points.
///
/// # Parameters
///
/// * `stdout`: The handle to the terminal
/// * `projected`: The projected vertices of the cube
/// * `vertices`: The vertices of the face to draw
/// * `shade_char`: The character to use for filling the face
/// * `color`: The color of the face
/// * `width`: The width of the terminal
/// * `height`: The height of the terminal
///
/// # Returns
///
/// A Result containing a () if the operation was successful, or an error
/// if something went wrong.
fn fill_face(buffer: &mut Buffer, projected: &[Point2D], vertices: &[usize], shade_char: char, color: Color, width: u16, height: u16) {
    let points: Vec<Point2D> = vertices.iter().map(|&i| projected[i]).collect();
    let points_with_wrap: Vec<Point2D> = points.iter().chain(points.first()).cloned().collect();

    for y in 0..height {
        let mut intersections = Vec::new();
        for window in points_with_wrap.windows(2) {
            if let Some(x) = edge_intersect(window[0], window[1], y) {
                intersections.push(x);
            }
        }
        intersections.sort_unstable();

        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                let start = chunk[0].max(0).min(width as i32 - 1) as usize;
                let end = chunk[1].max(0).min(width as i32 - 1) as usize;
                for x in start..=end {
                    buffer.set(x, y as usize, shade_char, color);
                }
            }
        }
    }
}

/// Calculates the center of a face
///
/// This function takes a list of 3D points and a list of vertex indices
/// that make up a face, and returns the center of the face. The center is
/// calculated by summing the coordinates of all the vertices, and then
/// dividing by the number of vertices.
///
/// # Parameters
///
/// * `points`: A list of 3D points
/// * `vertices`: A list of indices of the vertices of the face
///
/// # Returns
///
/// The center of the face as a 3D point
fn face_center(points: &[Point3D], vertices: &[usize]) -> Point3D {
    let mut center = Point3D { x: 0.0, y: 0.0, z: 0.0 };
    for &i in vertices {
        center.x += points[i].x;
        center.y += points[i].y;
        center.z += points[i].z;
    }
    let len = vertices.len() as f32;
    Point3D {
        x: center.x / len,
        y: center.y / len,
        z: center.z / len,
    }
}

/// Normalizes a 3D vector
///
/// This function takes a 3D vector and returns a new vector that has the same
/// direction, but with a length of 1. This is useful for calculating the dot
/// product of vectors, which is used in the lighting calculations.
///
/// # Parameters
///
/// * `v`: The 3D vector to normalize
///
/// # Returns
///
/// A normalized 3D vector
fn normalize(v: &Point3D) -> Point3D {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    Point3D {
        x: v.x / length,
        y: v.y / length,
        z: v.z / length,
    }
}

/// Calculates the dot product of two 3D vectors
///
/// The dot product of two vectors is a scalar value that is the sum of the products of the corresponding
/// components of the vectors. It is used in the lighting calculations to determine the amount of light
/// that is reflected from the surface of the cube.
///
/// # Parameters
///
/// * `a`: The first 3D vector
/// * `b`: The second 3D vector
///
/// # Returns
///
/// The dot product of `a` and `b`
fn dot_product(a: &Point3D, b: &Point3D) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Returns a character representing the shading intensity
///
/// The returned character is a Unicode character that represents a block with a certain amount of
/// shading. The characters are chosen so that the amount of shading increases as the value of
/// `shade` increases.
///
/// # Parameters
///
/// * `shade`: A value between 0 and 1 that represents the amount of shading to use
///
/// # Returns
///
/// A character representing the shading intensity
fn get_shade_char(shade: f32) -> char {
    let shade_chars = ['░', '▒', '▓', '█'];
    let index = ((shade * (shade_chars.len() - 1) as f32).round() as usize).min(shade_chars.len() - 1);
    shade_chars[index]
}

/// Creates a new color by applying a shade to the given color.
///
/// The shade is a value between 0 and 1 that represents the amount of shading to use.
/// The returned color will be darker than the original color if the shade value is less
/// than 1, and lighter if the shade value is greater than 1.
///
/// # Parameters
///
/// * `color`: The color to shade
/// * `shade`: The shade value to use
///
/// # Returns
///
/// A new color that is the original color with the specified shade applied
fn shade_color(color: &Color, shade: f32) -> Color {
    match color {
        Color::Rgb { r, g, b } => {
            // Apply the shade to the individual components of the color
            let r = (*r as f32 * shade) as u8;
            let g = (*g as f32 * shade) as u8;
            let b = (*b as f32 * shade) as u8;

            // Create a new color with the shaded components
            Color::Rgb { r, g, b }
        },
        _ => {
            // Apply the shade to the intensity of the color
            let intensity = (shade * 255.0) as u8;

            // Create a new color with the shaded intensity
            match color {
                Color::Red => Color::Rgb { r: intensity, g: 0, b: 0 },
                Color::Green => Color::Rgb { r: 0, g: intensity, b: 0 },
                Color::Blue => Color::Rgb { r: 0, g: 0, b: intensity },
                Color::Yellow => Color::Rgb { r: intensity, g: intensity, b: 0 },
                Color::Magenta => Color::Rgb { r: intensity, g: 0, b: intensity },
                Color::Cyan => Color::Rgb { r: 0, g: intensity, b: intensity },
                _ => Color::Rgb { r: intensity, g: intensity, b: intensity },
            }
        }
    }
}

/// Calculates the intersection of an edge with a horizontal line
///
/// This function takes two points `p1` and `p2` that represent an edge, and a
/// y-coordinate `y` that represents a horizontal line. It returns the x-coordinate
/// of the intersection between the edge and the line, if the intersection is
/// inside the edge. If the intersection is outside the edge, or if the edge is
/// vertical (i.e. `p1.y == p2.y`), the function returns `None`.
///
/// # Parameters
///
/// * `p1`: The first point of the edge
/// * `p2`: The second point of the edge
/// * `y`: The y-coordinate of the horizontal line
///
/// # Returns
///
/// The x-coordinate of the intersection between the edge and the line, or `None` if
/// the intersection is outside the edge or if the edge is vertical
fn edge_intersect(p1: Point2D, p2: Point2D, y: u16) -> Option<i32> {
    let y = y as i32;
    // Check if the intersection is inside the edge
    if (p1.y > y && p2.y <= y) || (p2.y > y && p1.y <= y) {
        // Calculate the x-coordinate of the intersection
        let x = p1.x + (p2.x - p1.x) * (y - p1.y) / (p2.y - p1.y);
        Some(x)
    } else {
        None
    }
}
