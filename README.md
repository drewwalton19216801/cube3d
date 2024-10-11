# cube3d

A 3D cube renderer with per-pixel lighting using Rust and the Druid GUI framework.

## Overview

`cube3d` is a Rust application that renders a rotating 3D cube with per-pixel lighting for accurate shading effects. It demonstrates fundamental 3D graphics concepts such as transformation matrices, rasterization, depth buffering, and lighting calculations without relying on a dedicated graphics library like OpenGL or Vulkan.

## Features

- **Per-Pixel Lighting:** Implements per-pixel lighting for realistic shading across the cube's surface.
- **Rotating 3D Cube:** Continuously rotates a 3D cube around its axis.
- **Debug Mode:** Displays frames per second (FPS), rotation angle, light position, and program information.
- **Mouse Zoom:** Zoom the cube in and out using the mouse wheel.

## Prerequisites

- **Rust:** Make sure you have Rust installed. You can download it from [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Cargo:** Comes bundled with Rust for package management and building.

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

Run the application with Cargo:

```bash
cargo run --release
```
## Enabling Debug Mode

To enable debug mode and display additional information, press the `d` key during program operation.

## Pausing/Resuming

To pause/resume the program, press the `p` key during program operation.

## Quitting

To quit the program, press the `q` key during program operation.

## How It Works
* **3D Transformations:** Applies rotation matrices to simulate cube rotation around the X and Y axes.
* **Rasterization:** Converts 3D triangles into pixels on the 2D screen.
* **Depth Buffering:** Implements a Z-buffer to handle occlusion of faces.
* **Per-Pixel Lighting:** Calculates lighting at each pixel by interpolating normals and positions, providing smooth shading.

## Dependencies

The project uses the following crates:

* `druid`: A data-first Rust-native UI design toolkit.

These dependencies are specified in `Cargo.toml` and will be automatically fetched when you build the project.

## Learning Resources

* **Druid Documentation:** [Druid Book](https://linebender.org/druid/)
* **Rust Programming Language:** [Rust Book](https://doc.rust-lang.org/book/)
* **3D Graphics Basics:** [Learn OpenGL](https://learnopengl.com/)

## Contributing

Contributions are welcome! Feel free to submit a pull request or open an issue for suggestions and improvements.

## License

This project is licensed under the MIT License. See the [License](https://github.com/drewwalton19216801/cube3d/blob/dev/LICENSE.md) file for details.

## Acknowledgements

* Inspired by basic 3D rendering techniques.
* Special thanks to the Rust community for their excellent resources.
* Claude 3.5 Sonnet for the initial implementations.
* GPT o1-preview for further improvements.
