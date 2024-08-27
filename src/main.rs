use std::io::{stdout, Write, Result};
use std::time::Duration;
use std::{thread, f32::consts::PI};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    cursor::MoveTo,
};
use termsize;

const DISTANCE: f32 = 50.0;
const ANGLE_INCREMENT: f32 = 0.05;

struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

struct Point2D {
    x: i32,
    y: i32,
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    let mut angle_x = 0.0;
    let mut angle_y = 0.0;

    loop {
        let (width, height) = get_terminal_size();
        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let cube_size = (width.min(height) as f32 * 0.4).max(10.0); // Use 40% of the smaller dimension, with a minimum size of 10

        execute!(stdout, Clear(ClearType::All))?;

        let cube = create_cube(cube_size);
        let rotated_cube = rotate_cube(&cube, angle_x, angle_y);
        let projected_cube = project_cube(&rotated_cube, center_x, center_y);

        draw_cube(&mut stdout, &projected_cube)?;

        execute!(stdout, MoveTo(0, 0), Print("Press Ctrl+C to exit"))?;
        stdout.flush()?;

        thread::sleep(Duration::from_millis(50));
        angle_x += ANGLE_INCREMENT;
        angle_y += ANGLE_INCREMENT * 0.7; // Rotate Y axis slightly slower for visual interest
        if angle_x >= 2.0 * PI {
            angle_x -= 2.0 * PI;
        }
        if angle_y >= 2.0 * PI {
            angle_y -= 2.0 * PI;
        }
    }
}

fn get_terminal_size() -> (u16, u16) {
    termsize::get()
        .map(|size| (size.cols, size.rows))
        .unwrap_or((80, 24))
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

fn draw_cube(stdout: &mut std::io::Stdout, cube: &[Point2D]) -> Result<()> {
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];

    for (start, end) in edges.iter() {
        draw_line(stdout, &cube[*start], &cube[*end])?;
    }

    Ok(())
}

fn draw_line(stdout: &mut std::io::Stdout, start: &Point2D, end: &Point2D) -> Result<()> {
    let dx = (end.x - start.x).abs();
    let dy = (end.y - start.y).abs();
    let sx = if start.x < end.x { 1 } else { -1 };
    let sy = if start.y < end.y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = start.x;
    let mut y = start.y;

    loop {
        execute!(
            stdout,
            MoveTo(x as u16, y as u16),
            SetForegroundColor(Color::Green),
            Print("*"),
            ResetColor
        )?;

        if x == end.x && y == end.y {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    Ok(())
}
