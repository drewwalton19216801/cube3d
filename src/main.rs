use std::io::{stdout, Write, Result};
use std::time::Duration;
use std::{thread, f32::consts::PI};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, size},
    cursor::MoveTo,
};

const DISTANCE: f32 = 50.0;
const ANGLE_INCREMENT: f32 = 0.05;
const MIN_CUBE_SIZE: f32 = 4.0;

#[derive(Clone, Copy)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct Point2D {
    x: i32,
    y: i32,
}

struct Face {
    vertices: [usize; 4],
    normal: Point3D,
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    let mut angle_x = 0.0;
    let mut angle_y = 0.0;

    loop {
        let (width, height) = get_terminal_size();
        if width < 10 || height < 10 {
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0), Print("Terminal too small"))?;
            stdout.flush()?;
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let cube_size = (width.min(height) as f32 * 0.4).max(MIN_CUBE_SIZE);

        execute!(stdout, Clear(ClearType::All))?;

        let cube = create_cube(cube_size);
        let faces = create_faces();
        let rotated_cube = rotate_cube(&cube, angle_x, angle_y);
        let projected_cube = project_cube(&rotated_cube, center_x, center_y);

        draw_cube(&mut stdout, &projected_cube, &rotated_cube, &faces, width, height)?;

        execute!(stdout, MoveTo(0, 0), Print("Press Ctrl+C to exit"))?;
        stdout.flush()?;

        thread::sleep(Duration::from_millis(50));
        angle_x += ANGLE_INCREMENT;
        angle_y += ANGLE_INCREMENT * 0.7;
        if angle_x >= 2.0 * PI {
            angle_x -= 2.0 * PI;
        }
        if angle_y >= 2.0 * PI {
            angle_y -= 2.0 * PI;
        }
    }
}

fn get_terminal_size() -> (u16, u16) {
    size().unwrap_or((80, 24))
}

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

fn create_faces() -> Vec<Face> {
    vec![
        Face { vertices: [0, 1, 2, 3], normal: Point3D { x: 0.0, y: 0.0, z: -1.0 } }, // Front
        Face { vertices: [5, 4, 7, 6], normal: Point3D { x: 0.0, y: 0.0, z: 1.0 } },  // Back
        Face { vertices: [1, 5, 6, 2], normal: Point3D { x: 1.0, y: 0.0, z: 0.0 } },  // Right
        Face { vertices: [4, 0, 3, 7], normal: Point3D { x: -1.0, y: 0.0, z: 0.0 } }, // Left
        Face { vertices: [3, 2, 6, 7], normal: Point3D { x: 0.0, y: 1.0, z: 0.0 } },  // Top
        Face { vertices: [1, 0, 4, 5], normal: Point3D { x: 0.0, y: -1.0, z: 0.0 } }, // Bottom
    ]
}

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

fn project_cube(cube: &[Point3D], center_x: i32, center_y: i32) -> Vec<Point2D> {
    cube.iter()
        .map(|p| {
            let x = (p.x * DISTANCE / (p.z + DISTANCE)) as i32 + center_x;
            let y = (p.y * DISTANCE / (p.z + DISTANCE)) as i32 + center_y;
            Point2D { x, y }
        })
        .collect()
}


fn draw_cube(stdout: &mut std::io::Stdout, projected: &[Point2D], rotated: &[Point3D], faces: &[Face], width: u16, height: u16) -> Result<()> {
    let light_direction = normalize(&Point3D { x: -1.0, y: -1.0, z: -1.0 });
    
    let mut face_depths: Vec<(usize, f32)> = faces.iter().enumerate()
        .map(|(i, face)| {
            let center = face_center(rotated, &face.vertices);
            (i, center.z)
        })
        .collect();
    
    face_depths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (face_index, _) in face_depths {
        let face = &faces[face_index];
        let rotated_normal = rotate_vector(&face.normal, rotated[face.vertices[0]], rotated[face.vertices[1]]);
        let shade = dot_product(&rotated_normal, &light_direction).max(0.1);

        let shade_char = get_shade_char(shade);
        let color = get_shade_color(shade);

        fill_face(stdout, projected, &face.vertices, shade_char, color, width, height)?;
    }

    Ok(())
}

fn rotate_vector(v: &Point3D, p1: Point3D, p2: Point3D) -> Point3D {
    let axis = normalize(&Point3D {
        x: p2.x - p1.x,
        y: p2.y - p1.y,
        z: p2.z - p1.z,
    });
    
    let cos_theta = dot_product(v, &axis);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let u = cross_product(v, &axis);
    let w = cross_product(&axis, &u);

    Point3D {
        x: cos_theta * v.x + sin_theta * w.x,
        y: cos_theta * v.y + sin_theta * w.y,
        z: cos_theta * v.z + sin_theta * w.z,
    }
}

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

fn normalize(v: &Point3D) -> Point3D {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    Point3D {
        x: v.x / length,
        y: v.y / length,
        z: v.z / length,
    }
}

fn dot_product(a: &Point3D, b: &Point3D) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross_product(a: &Point3D, b: &Point3D) -> Point3D {
    Point3D {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn get_shade_char(shade: f32) -> char {
    let shade_chars = ['░', '▒', '▓', '█'];
    let index = ((shade * (shade_chars.len() - 1) as f32).round() as usize).min(shade_chars.len() - 1);
    shade_chars[index]
}

fn get_shade_color(shade: f32) -> Color {
    let intensity = (shade * 255.0) as u8;
    Color::Rgb { r: intensity, g: intensity, b: intensity }
}

fn fill_face(stdout: &mut std::io::Stdout, projected: &[Point2D], vertices: &[usize], shade_char: char, color: Color, width: u16, height: u16) -> Result<()> {
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
                let start = chunk[0].max(0).min(width as i32 - 1) as u16;
                let end = chunk[1].max(0).min(width as i32 - 1) as u16;
                for x in start..=end {
                    execute!(
                        stdout,
                        MoveTo(x, y),
                        SetForegroundColor(color),
                        Print(shade_char),
                        ResetColor
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn edge_intersect(p1: Point2D, p2: Point2D, y: u16) -> Option<i32> {
    let y = y as i32;
    if (p1.y > y && p2.y <= y) || (p2.y > y && p1.y <= y) {
        let x = p1.x + (p2.x - p1.x) * (y - p1.y) / (p2.y - p1.y);
        Some(x)
    } else {
        None
    }
}