use druid::Color;

/// Edge function used in rasterization
#[inline(always)]
pub fn edge_function(a: &[f32; 2], b: &[f32; 2], c: &[f32; 2]) -> f32 {
    (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])
}

/// Checks if a point is inside a triangle using the edge function with early exit
#[inline(always)]
pub fn point_in_triangle(p: &[f32; 2], a: &[f32; 2], b: &[f32; 2], c: &[f32; 2]) -> bool {
    let w0 = edge_function(b, c, p);
    if w0 < 0.0 {
        let w1 = edge_function(c, a, p);
        if w1 < 0.0 {
            let w2 = edge_function(a, b, p);
            return w2 < 0.0;
        }
        return false;
    } else {
        let w1 = edge_function(c, a, p);
        if w1 < 0.0 {
            return false;
        }
        let w2 = edge_function(a, b, p);
        return w2 >= 0.0;
    }
}

/// Multiplies a 3x3 matrix by a 3-dimensional vector (unrolled for performance)
#[inline(always)]
pub fn multiply_matrix_vector(matrix: &[[f32; 3]; 3], vector: &[f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * vector[0] + matrix[0][1] * vector[1] + matrix[0][2] * vector[2],
        matrix[1][0] * vector[0] + matrix[1][1] * vector[1] + matrix[1][2] * vector[2],
        matrix[2][0] * vector[0] + matrix[2][1] * vector[1] + matrix[2][2] * vector[2],
    ]
}

/// Multiplies two 3x3 matrices (unrolled for performance)
#[inline(always)]
pub fn multiply_matrices(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0] + a[0][2] * b[2][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1] + a[0][2] * b[2][1],
            a[0][0] * b[0][2] + a[0][1] * b[1][2] + a[0][2] * b[2][2],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0] + a[1][2] * b[2][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1] + a[1][2] * b[2][1],
            a[1][0] * b[0][2] + a[1][1] * b[1][2] + a[1][2] * b[2][2],
        ],
        [
            a[2][0] * b[0][0] + a[2][1] * b[1][0] + a[2][2] * b[2][0],
            a[2][0] * b[0][1] + a[2][1] * b[1][1] + a[2][2] * b[2][1],
            a[2][0] * b[0][2] + a[2][1] * b[1][2] + a[2][2] * b[2][2],
        ],
    ]
}

/// Fast inverse square root approximation (Quake III algorithm)
#[inline(always)]
fn fast_inv_sqrt(x: f32) -> f32 {
    let x2 = x * 0.5;
    let mut i = x.to_bits();
    i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    y * (1.5 - x2 * y * y)
}

/// Calculates the normal vector of a triangle with fast normalization
#[inline(always)]
pub fn calculate_normal(a: &[f32; 3], b: &[f32; 3], c: &[f32; 3]) -> [f32; 3] {
    let ux = b[0] - a[0];
    let uy = b[1] - a[1];
    let uz = b[2] - a[2];
    let vx = c[0] - a[0];
    let vy = c[1] - a[1];
    let vz = c[2] - a[2];
    
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;
    
    let length_sq = nx * nx + ny * ny + nz * nz;
    let inv_length = fast_inv_sqrt(length_sq);
    
    [nx * inv_length, ny * inv_length, nz * inv_length]
}

/// Calculates the light intensity based on the normal vector and light position
#[inline(always)]
pub fn calculate_light_intensity(
    normal: &[f32; 3],
    position: &[f32; 3],
    light_pos: &[f32; 3],
) -> f32 {
    let dx = light_pos[0] - position[0];
    let dy = light_pos[1] - position[1];
    let dz = light_pos[2] - position[2];
    
    let length_sq = dx * dx + dy * dy + dz * dz;
    let inv_length = fast_inv_sqrt(length_sq);
    
    let dot_product = (normal[0] * dx + normal[1] * dy + normal[2] * dz) * inv_length;
    dot_product.max(0.1) // Ensure a minimum ambient light
}

/// Applies lighting to a color (optimized with bit manipulation)
#[inline(always)]
pub fn apply_lighting(color: Color, intensity: f32) -> Color {
    let rgba = color.as_rgba8();
    let r = ((rgba.0 as f32 * intensity).min(255.0) + 0.5) as u8;
    let g = ((rgba.1 as f32 * intensity).min(255.0) + 0.5) as u8;
    let b = ((rgba.2 as f32 * intensity).min(255.0) + 0.5) as u8;
    Color::rgb8(r, g, b)
}

/// Vector length squared (faster than length when you only need to compare)
#[inline(always)]
pub fn length_squared(v: &[f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// Vector normalization using fast inverse square root
#[inline(always)]
pub fn normalize(v: &[f32; 3]) -> [f32; 3] {
    let length_sq = length_squared(v);
    let inv_length = fast_inv_sqrt(length_sq);
    [v[0] * inv_length, v[1] * inv_length, v[2] * inv_length]
}