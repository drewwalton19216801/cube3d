use std::error::Error;
use std::f32::consts::PI;
use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

const DISTANCE: f32 = 5.0;
const ANGLE_INCREMENT: f32 = 0.05;
const CUBE_SIZE: f32 = 2.0;

#[derive(Clone, Copy)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct Point2D {
    x: f32,
    y: f32,
}

struct Face {
    vertices: [usize; 4],
}

struct App {
    cube: Vec<Point3D>,
    faces: Vec<Face>,
    angle_x: f32,
    angle_y: f32,
}

impl App {
    fn new() -> Self {
        let cube = create_cube(CUBE_SIZE);
        let faces = create_faces();
        Self {
            cube,
            faces,
            angle_x: 0.0,
            angle_y: 0.0,
        }
    }

    fn on_tick(&mut self) {
        self.angle_x += ANGLE_INCREMENT;
        self.angle_y += ANGLE_INCREMENT * 0.7;
        if self.angle_x >= 2.0 * PI {
            self.angle_x -= 2.0 * PI;
        }
        if self.angle_y >= 2.0 * PI {
            self.angle_y -= 2.0 * PI;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(33);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::<B>(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame, app: &App) {
    let size = f.area();

    let block = Block::default()
        .title("3D Cube")
        .borders(Borders::ALL);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(size);

    let rotated_cube = rotate_cube(&app.cube, app.angle_x, app.angle_y);
    let projected_cube = project_cube(&rotated_cube);
    let render_data = prepare_render_data(&projected_cube, &app.faces, size);

    let cube_render = Paragraph::new(render_data)
        .style(Style::default().fg(Color::White))
        .block(Block::default());

    f.render_widget(cube_render, chunks[0]);
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
        Face { vertices: [0, 1, 2, 3]}, // Front
        Face { vertices: [5, 4, 7, 6]},  // Back
        Face { vertices: [1, 5, 6, 2]},  // Right
        Face { vertices: [4, 0, 3, 7]}, // Left
        Face { vertices: [3, 2, 6, 7]},  // Top
        Face { vertices: [1, 0, 4, 5]}, // Bottom
    ]
}

fn rotate_cube(cube: &[Point3D], angle_x: f32, angle_y: f32) -> Vec<Point3D> {
    cube.iter()
        .map(|p| rotate_point(p, angle_x, angle_y))
        .collect()
}

fn rotate_point(point: &Point3D, angle_x: f32, angle_y: f32) -> Point3D {
    let cos_x = angle_x.cos();
    let sin_x = angle_x.sin();
    let cos_y = angle_y.cos();
    let sin_y = angle_y.sin();

    let y1 = point.y * cos_x - point.z * sin_x;
    let z1 = point.y * sin_x + point.z * cos_x;

    let x2 = point.x * cos_y + z1 * sin_y;
    let z2 = -point.x * sin_y + z1 * cos_y;

    Point3D { x: x2, y: y1, z: z2 }
}

fn project_cube(cube: &[Point3D]) -> Vec<Point2D> {
    cube.iter()
        .map(|p| {
            let x = p.x * DISTANCE / (p.z + DISTANCE);
            let y = p.y * DISTANCE / (p.z + DISTANCE);
            Point2D { x, y }
        })
        .collect()
}

fn prepare_render_data(projected: &[Point2D], faces: &[Face], size: ratatui::layout::Rect) -> Vec<Line<'static>> {
    let mut render_data = vec![Line::from(vec![Span::raw(" ".repeat(size.width as usize))]); size.height as usize];
    let center_x = size.width as f32 / 2.0;
    let center_y = size.height as f32 / 2.0;
    let scale = size.height as f32 / 4.0;

    for face in faces {
        let points: Vec<(i16, i16)> = face.vertices.iter()
            .map(|&i| {
                let x = (projected[i].x * scale + center_x) as i16;
                let y = (projected[i].y * scale + center_y) as i16;
                (x, y)
            })
            .collect();

        for i in 0..4 {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % 4];
            draw_line(&mut render_data, x1, y1, x2, y2, size);
        }
    }

    render_data
}

fn draw_line(render_data: &mut [Line<'static>], mut x1: i16, mut y1: i16, x2: i16, y2: i16, size: ratatui::layout::Rect) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x1 >= 0 && x1 < size.width as i16 && y1 >= 0 && y1 < size.height as i16 {
            let line = &mut render_data[y1 as usize];
            let content = line.spans[0].content.to_string();
            let mut new_content = content.chars().collect::<Vec<char>>();
            if let Some(c) = new_content.get_mut(x1 as usize) {
                if *c == ' ' {
                    *c = '█';
                }
            }
            line.spans[0] = Span::raw(new_content.iter().collect::<String>());
        }

        if x1 == x2 && y1 == y2 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }
}