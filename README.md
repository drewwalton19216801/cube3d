# cube3d

[![Rust](https://github.com/drewwalton19216801/cube3d/actions/workflows/rust.yml/badge.svg)](https://github.com/drewwalton19216801/cube3d/actions/workflows/rust.yml)

A high-performance 3D cube renderer with per-pixel lighting using Rust and the Druid GUI framework.

## Overview

`cube3d` is a Rust application that renders a rotating 3D cube with per-pixel lighting for accurate shading effects. It demonstrates fundamental 3D graphics concepts such as transformation matrices, rasterization, depth buffering, and lighting calculations without relying on a dedicated graphics library like OpenGL or Vulkan. The renderer is optimized for performance with mathematical optimizations and efficient algorithms.

## Features

- **Per-Pixel Lighting:** Implements per-pixel lighting for realistic shading across the cube's surface.
- **World Space Lighting:** Displays the light position in world space.
- **Wireframe Mode:** Renders the cube in wireframe mode.
- **Rotating 3D Cube:** Continuously rotates a 3D cube around its axis.
- **Debug Mode:** Displays frames per second (FPS), rotation angle, light position, and program information.
- **Mouse Zoom:** Zoom the cube in and out using the mouse wheel.
- **Mouse Rotation:** Rotate the cube around its axis using the mouse.
- **Mouse Translation:** Translate the cube using the mouse.
- **Performance Optimized:** Uses f32 precision, fast inverse square root, and optimized mathematical operations.

## Performance Optimizations

The latest version includes significant performance improvements:

- **f32 Precision:** Switched from f64 to f32 for 2-4x faster floating-point operations
- **SIMD Operations:** Uses the `glam` library for hardware-accelerated vector and matrix operations
- **Function Inlining:** All mathematical functions are inlined to eliminate call overhead
- **Parallel Rasterization:** Uses `rayon` for multi-threaded triangle rendering
- **Triangle Rasterization:** Early exit point-in-triangle test with optimized culling
- **Memory Optimization:** Reduced allocations and improved cache locality
- **Pre-allocated Buffers:** Pixel and Z-buffers are pre-allocated and reused across frames

## Prerequisites

- **Rust:** Make sure you have Rust installed. You can download it from [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Cargo:** Comes bundled with Rust for package management and building.
- **Linux only:** On Linux (and probably other *NIX platforms other than macOS), GTK+3 is required. On Ubuntu-based distros, `sudo apt-get install libgtk-3-dev` will suffice.

## Installing

Install cube3d with cargo:

```bash
cargo install cube3d
```

## Building

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/drewwalton19216801/cube3d.git
cd cube3d
```

Build the project using Cargo:

```bash
cargo build --release
```

This will compile the project in release mode for optimal performance.

## Running

Run the application with Cargo, or if you installed it above, run directly:

```bash
cargo run --release
```

```bash
cube3d
```

## Controls

### Keyboard Controls

- **H**: Display help information
- **D**: Toggle debug mode (shows FPS, angles, light position)
- **P/Space**: Pause/unpause rotation
- **W**: Toggle wireframe mode
- **R**: Reset cube position and zoom
- **Q**: Quit the application

### Mouse Controls

- **Left Click + Drag**: Rotate the cube
- **Right Click + Drag**: Translate the cube
- **Mouse Wheel**: Zoom in/out

## How It Works

### Core Rendering Pipeline

- **3D Transformations:** Applies rotation matrices to simulate cube rotation around the X and Y axes.

- **Rasterization:** Converts 3D triangles into pixels on the 2D screen using optimized edge functions.
- **Depth Buffering:** Implements a Z-buffer to handle occlusion of faces.
- **Per-Pixel Lighting:** Calculates lighting at each pixel by interpolating normals and positions, providing smooth shading.

### Performance Features

- **Optimized Math:** Uses f32 precision with inlined mathematical functions and SIMD operations via `glam`

- **Parallel Rendering:** Multi-threaded triangle rasterization using `rayon`
- **Efficient Culling:** Back-face, viewport, and depth culling with early exits
- **Memory Efficient:** Pre-allocated buffers and cache-friendly access patterns

## Dependencies

The project uses the following crates:

- `druid`: A data-first Rust-native UI design toolkit for the GUI
- `glam`: A simple and fast linear algebra library with SIMD support for vector and matrix operations
- `rayon`: A data-parallelism library for multi-threaded triangle rasterization

These dependencies are specified in [`Cargo.toml`](Cargo.toml) and will be automatically fetched when you build the project.

## Performance Benchmarks

The latest optimizations provide:

- **2-4x faster floating-point operations** (f32 vs f64)
- **Hardware-accelerated SIMD operations** (via `glam` library)
- **Multi-threaded rendering** (parallel triangle rasterization with `rayon`)
- **Significant reduction in function call overhead** (inlining)
- **Better CPU cache utilization** (pre-allocated buffers and memory access patterns)
- **Faster triangle rasterization** (early exits and optimized culling tests)
- **Reduced memory bandwidth** (smaller data types and buffer reuse)

## Compatibility

`cube3d` is cross-platform and has been tested on Linux and Windows, and should also work on macOS.

## Learning Resources

- **Druid Documentation:** [Druid Book](https://linebender.org/druid/)
- **Rust Programming Language:** [Rust Book](https://doc.rust-lang.org/book/)
- **3D Graphics Basics:** [Learn OpenGL](https://learnopengl.com/)
- **Performance Optimization:** [Rust Performance Book](https://nnethercote.github.io/perf-book/)

## Contributing

Contributions are welcome! Feel free to submit a pull request or open an issue for suggestions and improvements.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Test with `cargo test` and `cargo run --release`
5. Commit your changes: `git commit -m "feat: description"`
6. Push to your fork: `git push origin feature-name`
7. Create a pull request

## License

This project is licensed under the MIT License. See the [License](https://github.com/drewwalton19216801/cube3d/blob/dev/LICENSE.md) file for details.

## Acknowledgements

- Inspired by basic 3D rendering techniques
- Special thanks to the Rust community for their excellent resources
- The `glam` library for fast SIMD-accelerated linear algebra operations
- The `rayon` library for easy data parallelism
- Claude 3.5 Sonnet for the initial implementations
- GPT o1-preview for further improvements
- Cursor for further improvements and performance optimizations
