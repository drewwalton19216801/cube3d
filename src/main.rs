//! # 3D Cube Renderer for Terminal
//!
//! This program renders a rotating 3D cube in the terminal using ASCII characters
//! and ANSI colors. It demonstrates basic 3D graphics concepts such as:
//!
//! - 3D to 2D projection
//! - Rotation in 3D space
//! - Face culling
//! - Simple lighting and shading
//!
//! The cube rotates continuously, with each face colored differently and shaded
//! based on its orientation relative to a light source. The program handles
//! terminal resizing and provides a smooth animation at approximately 30 FPS.
//!
//! ## Features:
//! - Real-time 3D rendering in the terminal
//! - Colored cube faces with dynamic shading
//! - Smooth rotation animation
//! - Responsive to terminal resizing
//! - Simple user interface (press 'q' to quit)
//!
//! ## Dependencies:
//! - crossterm: For terminal manipulation and event handling
//!
//! ## Usage:
//! Run the program and watch the cube rotate. Press 'q' or 'Esc' to exit.
//! The cube will automatically adjust its size based on the terminal dimensions.
//!
use clap::Parser;
use crossterm::style::Print;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use std::io::{stdout, Result, Write};
use std::time::{Duration, Instant};
use std::{f32::consts::PI, thread};

// Constants for cube rendering and animation
const DISTANCE: f32 = 50.0;
const ANGLE_INCREMENT: f32 = 0.05;
const MIN_CUBE_SIZE: f32 = 4.0;
const LIGHT_DIRECTION: Point3D = Point3D {
    x: -1.0,
    y: -1.0,
    z: -1.0,
};
const FRAME_DURATION: Duration = Duration::from_millis(33); // ~30 FPS
const CAMERA_BOB_SPEED: f32 = 0.05;
const CAMERA_BOB_AMPLITUDE: f32 = 10.0;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

/// Represents a point in 3D space
#[derive(Clone, Copy)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

/// Represents a point in 2D space (for projection)
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

/// Represents the camera position
struct Camera {
    x: f32,
    y: f32,
    start_time: Instant,
}

impl Camera {
    fn new() -> Self {
        Camera {
            x: 0.0,
            y: 0.0,
            start_time: Instant::now(),
        }
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.x = CAMERA_BOB_AMPLITUDE * (elapsed * CAMERA_BOB_SPEED).sin();
        self.y = CAMERA_BOB_AMPLITUDE * (elapsed * CAMERA_BOB_SPEED * 0.5).cos();
    }
}

/// Buffer for storing and rendering the cube
struct Buffer {
    width: usize,
    height: usize,
    content: Vec<Vec<(char, Color)>>,
}

impl Buffer {
    /// Creates a new buffer with the specified dimensions
    fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            content: vec![vec![(' ', Color::Reset); width]; height],
        }
    }

    /// Sets a character and color at the specified position in the buffer
    fn set(&mut self, x: usize, y: usize, ch: char, color: Color) {
        if x < self.width && y < self.height {
            self.content[y][x] = (ch, color);
        }
    }

    /// Clears the buffer, resetting all characters and colors
    fn clear(&mut self) {
        for row in self.content.iter_mut() {
            for cell in row.iter_mut() {
                *cell = (' ', Color::Reset);
            }
        }
    }

    /// Resizes the buffer to new dimensions
    fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.content = vec![vec![(' ', Color::Reset); new_width]; new_height];
    }

    /// Renders the buffer content to the terminal
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

/// Main function to run the 3D cube animation
fn main() -> Result<()> {
    let args = Args::parse();

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
    let resize_cooldown = Duration::from_millis(500);

    let mut camera = Camera::new();

    draw_welcome_message(&mut buffer, width, height);
    buffer.render(&mut stdout)?;

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
                }
                Event::Resize(new_width, new_height) => {
                    width = new_width;
                    height = new_height;
                    buffer.resize(width as usize, height as usize);
                    execute!(stdout, Clear(ClearType::All))?;
                    is_resizing = true;
                    last_resize_time = Instant::now();
                }
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
                execute!(
                    stdout,
                    Clear(ClearType::All),
                    MoveTo(0, 0),
                    Print("Terminal too small")
                )?;
                stdout.flush()?;
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            let center_x = width as i32 / 2;
            let center_y = height as i32 / 2;
            let cube_size = (width.min(height) as f32 * 0.4).max(MIN_CUBE_SIZE);

            let cube = create_cube(cube_size);
            let faces = create_faces();

            // Update camera position
            camera.update();

            // Apply camera offset to cube before rotation
            let offset_cube: Vec<Point3D> = cube
                .iter()
                .map(|p| Point3D {
                    x: p.x + camera.x,
                    y: p.y + camera.y,
                    z: p.z,
                })
                .collect();

            let rotated_cube = rotate_cube(&offset_cube, angle_x, angle_y);
            let projected_cube = project_cube(&rotated_cube, center_x, center_y);

            buffer.clear();
            draw_cube(
                &mut buffer,
                &projected_cube,
                &rotated_cube,
                &faces,
                &light_direction,
                angle_x,
                angle_y,
                width,
                height,
            )?;

            // Debug output for camera position
            if args.debug {
                let debug_message = format!("Camera: x={:.2}, y={:.2}", camera.x, camera.y);
                for (i, ch) in debug_message.chars().enumerate() {
                    buffer.set(i, 0, ch, Color::White);
                }
            }

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

/// Draws a welcome message in the center of the buffer
fn draw_welcome_message(buffer: &mut Buffer, width: u16, height: u16) {
    let welcome_message = "Welcome to cube3d, press 'q' to quit";
    let x = width as usize / 2 - welcome_message.len() / 2;
    let y = height as usize / 2;

    for (i, ch) in welcome_message.chars().enumerate() {
        buffer.set(x + i, y, ch, Color::White);
    }
}

/// Draws a resizing message in the center of the buffer
fn draw_resize_message(buffer: &mut Buffer, width: u16, height: u16) {
    let message = "Resizing...";
    let x = width as usize / 2 - message.len() / 2;
    let y = height as usize / 2;

    for (i, ch) in message.chars().enumerate() {
        buffer.set(x + i, y, ch, Color::White);
    }
}

/// Creates the vertices of a cube with the given size
fn create_cube(size: f32) -> Vec<Point3D> {
    vec![
        Point3D {
            x: -size / 2.0,
            y: -size / 2.0,
            z: -size / 2.0,
        },
        Point3D {
            x: size / 2.0,
            y: -size / 2.0,
            z: -size / 2.0,
        },
        Point3D {
            x: size / 2.0,
            y: size / 2.0,
            z: -size / 2.0,
        },
        Point3D {
            x: -size / 2.0,
            y: size / 2.0,
            z: -size / 2.0,
        },
        Point3D {
            x: -size / 2.0,
            y: -size / 2.0,
            z: size / 2.0,
        },
        Point3D {
            x: size / 2.0,
            y: -size / 2.0,
            z: size / 2.0,
        },
        Point3D {
            x: size / 2.0,
            y: size / 2.0,
            z: size / 2.0,
        },
        Point3D {
            x: -size / 2.0,
            y: size / 2.0,
            z: size / 2.0,
        },
    ]
}

/// Creates the faces of the cube, defining vertices and colors
fn create_faces() -> Vec<Face> {
    vec![
        Face {
            vertices: [0, 1, 2, 3],
            normal: Point3D {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            color: Color::Red,
        },
        Face {
            vertices: [5, 4, 7, 6],
            normal: Point3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            color: Color::Green,
        },
        Face {
            vertices: [1, 5, 6, 2],
            normal: Point3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            color: Color::Blue,
        },
        Face {
            vertices: [4, 0, 3, 7],
            normal: Point3D {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            color: Color::Yellow,
        },
        Face {
            vertices: [3, 2, 6, 7],
            normal: Point3D {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            color: Color::Magenta,
        },
        Face {
            vertices: [1, 0, 4, 5],
            normal: Point3D {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            color: Color::Cyan,
        },
    ]
}

/// Rotates the cube vertices around the X and Y axes
fn rotate_cube(cube: &[Point3D], angle_x: f32, angle_y: f32) -> Vec<Point3D> {
    cube.iter()
        .map(|p| {
            // Rotate around X-axis
            let y1 = p.y * angle_x.cos() - p.z * angle_x.sin();
            let z1 = p.y * angle_x.sin() + p.z * angle_x.cos();

            // Rotate around Y-axis
            let x2 = p.x * angle_y.cos() + z1 * angle_y.sin();
            let z2 = -p.x * angle_y.sin() + z1 * angle_y.cos();

            Point3D {
                x: x2,
                y: y1,
                z: z2,
            }
        })
        .collect()
}

/// Projects 3D points onto a 2D plane for rendering
fn project_cube(cube: &[Point3D], center_x: i32, center_y: i32) -> Vec<Point2D> {
    cube.iter()
        .map(|p| {
            let x = (p.x * DISTANCE / (p.z + DISTANCE)) as i32 + center_x;
            let y = (p.y * DISTANCE / (p.z + DISTANCE)) as i32 + center_y;
            Point2D { x, y }
        })
        .collect()
}

/// Draws the cube on the buffer, applying rotation, projection, and shading
fn draw_cube(
    buffer: &mut Buffer,
    projected: &[Point2D],
    rotated: &[Point3D],
    faces: &[Face],
    light_direction: &Point3D,
    angle_x: f32,
    angle_y: f32,
    width: u16,
    height: u16,
) -> Result<()> {
    let mut face_depths: Vec<(usize, f32)> = faces
        .iter()
        .enumerate()
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

        fill_face(
            buffer,
            projected,
            &face.vertices,
            shade_char,
            color,
            width,
            height,
        );
    }

    Ok(())
}

/// Rotates a single point around the X and Y axes
fn rotate_point(point: &Point3D, angle_x: f32, angle_y: f32) -> Point3D {
    let cos_x = angle_x.cos();
    let sin_x = angle_x.sin();
    let cos_y = angle_y.cos();
    let sin_y = angle_y.sin();

    let y1 = point.y * cos_x - point.z * sin_x;
    let z1 = point.y * sin_x + point.z * cos_x;

    let x2 = point.x * cos_y + z1 * sin_y;
    let z2 = -point.x * sin_y + z1 * cos_y;

    Point3D {
        x: x2,
        y: y1,
        z: z2,
    }
}

/// Fills a face of the cube with the appropriate shading
fn fill_face(
    buffer: &mut Buffer,
    projected: &[Point2D],
    vertices: &[usize],
    shade_char: char,
    color: Color,
    width: u16,
    height: u16,
) {
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

/// Calculates the center point of a face
fn face_center(points: &[Point3D], vertices: &[usize]) -> Point3D {
    let mut center = Point3D {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
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
fn normalize(v: &Point3D) -> Point3D {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    Point3D {
        x: v.x / length,
        y: v.y / length,
        z: v.z / length,
    }
}

/// Calculates the dot product of two 3D vectors
fn dot_product(a: &Point3D, b: &Point3D) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Determines the appropriate shading character based on the light intensity
fn get_shade_char(shade: f32) -> char {
    let shade_chars = ['░', '▒', '▓', '█'];
    let index =
        ((shade * (shade_chars.len() - 1) as f32).round() as usize).min(shade_chars.len() - 1);
    shade_chars[index]
}

/// Adjusts the color based on the shading intensity
fn shade_color(color: &Color, shade: f32) -> Color {
    match color {
        Color::Rgb { r, g, b } => {
            let r = (*r as f32 * shade) as u8;
            let g = (*g as f32 * shade) as u8;
            let b = (*b as f32 * shade) as u8;

            Color::Rgb { r, g, b }
        }
        _ => {
            let intensity = (shade * 255.0) as u8;

            match color {
                Color::Red => Color::Rgb {
                    r: intensity,
                    g: 0,
                    b: 0,
                },
                Color::Green => Color::Rgb {
                    r: 0,
                    g: intensity,
                    b: 0,
                },
                Color::Blue => Color::Rgb {
                    r: 0,
                    g: 0,
                    b: intensity,
                },
                Color::Yellow => Color::Rgb {
                    r: intensity,
                    g: intensity,
                    b: 0,
                },
                Color::Magenta => Color::Rgb {
                    r: intensity,
                    g: 0,
                    b: intensity,
                },
                Color::Cyan => Color::Rgb {
                    r: 0,
                    g: intensity,
                    b: intensity,
                },
                _ => Color::Rgb {
                    r: intensity,
                    g: intensity,
                    b: intensity,
                },
            }
        }
    }
}

/// Calculates the intersection of an edge with a horizontal line
fn edge_intersect(p1: Point2D, p2: Point2D, y: u16) -> Option<i32> {
    let y = y as i32;
    if (p1.y > y && p2.y <= y) || (p2.y > y && p1.y <= y) {
        let x = p1.x + (p2.x - p1.x) * (y - p1.y) / (p2.y - p1.y);
        Some(x)
    } else {
        None
    }
}
